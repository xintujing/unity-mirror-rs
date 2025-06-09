use crate::commons::action::SelfMutAction;
use crate::commons::revel_arc::RevelArc;
use crate::commons::revel_weak::RevelWeak;
use crate::metadata_settings::metadata::Metadata;
use crate::metadata_settings::mirror::metadata_network_manager::MetadataNetworkManagerWrapper;
use crate::metadata_settings::mirror::metadata_network_root_manager::MetadataNetworkRootManager;
use crate::mirror::components::network_room_player::NetworkRoomPlayer;
use crate::mirror::transport::TransportError;
use crate::mirror::{
    NetworkConnection, NetworkConnectionToClient, NetworkIdentity, NetworkManager, NetworkServer,
    RemovePlayerOptions, ReplacePlayerOptions,
};
use crate::unity_engine::{GameObject, MonoBehaviour, WorldManager};
use std::any::Any;
use std::collections::HashSet;
use std::error::Error;
use unity_mirror_macro_rs::{action, namespace, network_manager, NetworkManagerFactory};

#[derive(Clone)]
pub struct PendingPlayer {
    pub connection: RevelWeak<Box<NetworkConnectionToClient>>,
    pub room_player: RevelWeak<GameObject>,
}

#[namespace(prefix = "Mirror")]
#[network_manager(parent(NetworkManager))]
pub struct NetworkRoomManager {
    // 最少可以自动启动游戏的玩家数量
    pub min_players: i32,
    // 预制可用于房间播放器
    pub room_player_prefab: String,
    // 用于房间的场景 这类似于 NetworkRoomManager 的 offline scene
    pub room_scene: String,
    // 从房间玩游戏的场景 这类似于 NetworkRoomManager 的online scene。
    pub gameplay_scene: String,
    // 房间里的玩家名单
    pub pending_players: Vec<PendingPlayer>,
    // 这些插槽跟踪进入房间的玩家。
    pub room_slots: HashSet<RevelWeak<Box<NetworkRoomPlayer>>>,
    // 诊断标志 表明所有玩家都准备好玩
    all_players_ready: bool,

    client_index: usize,

    pub on_room_start_server: SelfMutAction<(), ()>,
    pub on_room_stop_server: SelfMutAction<(), ()>,
    pub on_room_server_connect: SelfMutAction<(RevelArc<Box<NetworkConnectionToClient>>,), ()>,
    pub on_room_server_disconnect: SelfMutAction<(RevelArc<Box<NetworkConnectionToClient>>,), ()>,
    pub on_room_server_scene_changed: SelfMutAction<(String,), ()>,
    pub on_room_server_create_room_player:
        SelfMutAction<(RevelArc<Box<NetworkConnectionToClient>>,), Option<RevelArc<GameObject>>>,
    pub on_room_server_create_game_player: SelfMutAction<
        (
            RevelArc<Box<NetworkConnectionToClient>>,
            RevelArc<GameObject>,
        ),
        Option<RevelArc<GameObject>>,
    >,
    pub on_room_server_add_player: SelfMutAction<(RevelArc<Box<NetworkConnectionToClient>>,), ()>,
    pub on_room_server_scene_loaded_for_player: SelfMutAction<
        (
            RevelArc<Box<NetworkConnectionToClient>>,
            RevelArc<GameObject>,
            RevelArc<GameObject>,
        ),
        bool,
    >,
    pub ready_status_changed: SelfMutAction<(), ()>,
    pub on_room_server_players_ready: SelfMutAction<(), ()>,
    pub on_room_server_players_not_ready: SelfMutAction<(), ()>,

    pub on_room_client_enter: SelfMutAction<(), ()>,
}
impl NetworkRoomManager {
    fn set_all_players_ready(&mut self, now_ready: bool) {
        let was_ready = self.all_players_ready;
        if was_ready != now_ready {
            self.all_players_ready = now_ready;

            if now_ready {
                self.on_room_server_players_ready();
            } else {
                self.on_room_server_players_not_ready.call(());
            }
        }
    }
}

impl NetworkRoomManager {
    /// 新客户连接时在服务器上调用
    fn on_server_connect(&mut self, mut connection: RevelArc<Box<NetworkConnectionToClient>>) {
        println!("on_server_connect {}", connection.connection_id);
        if let Some(world) = WorldManager::active_world().upgrade() {
            if world.get_scene_path() != self.room_scene {
                log::info!(
                    "Not in Room scene...disconnecting {}",
                    connection.connection_id
                );
                connection.disconnect();
                return;
            }
        }

        self.parent.on_server_connect_default(connection.clone());
        self.on_room_server_connect.call((connection,));
    }

    /// 当客户端断开连接时在服务器上调用
    fn on_server_disconnect(&mut self, connection: RevelArc<Box<NetworkConnectionToClient>>) {
        if let Some(conn_identity) = connection.identity.upgrade() {
            if let Some(game_object) = conn_identity.game_object.upgrade() {
                if let Some(room_player) = game_object.try_get_component2::<NetworkRoomPlayer>() {
                    self.room_slots
                        .retain(|player| !player.ptr_eq(&room_player.downgrade()));

                    for client_owned_object in connection.owned.iter() {
                        if let Some(client_owned_object_game_object) =
                            client_owned_object.game_object.upgrade()
                        {
                            if let Some(room_player) = client_owned_object_game_object
                                .try_get_component2::<NetworkRoomPlayer>()
                            {
                                self.room_slots
                                    .retain(|player| !player.ptr_eq(&room_player.downgrade()));
                            }
                        }
                    }
                }
            }
        }

        self.set_all_players_ready(false);

        for player in self.room_slots.iter() {
            if let Some(player) = player.upgrade() {
                if let Some(player_game_object) = player.game_object.upgrade() {
                    if let Some(mut network_room_player) =
                        player_game_object.try_get_component2::<NetworkRoomPlayer>()
                    {
                        network_room_player.set_ready_to_begin(false)
                    }
                }
            }
        }

        if let Some(world) = WorldManager::active_world().upgrade() {
            if world.get_scene_path() == self.room_scene {
                self.recalculate_room_player_indices()
            }
        }

        self.on_room_server_disconnect.call((connection.clone(),));

        self.on_server_disconnect_default(connection)
    }

    /// 客户准备就绪时在服务器上打电话
    pub fn on_server_ready(&mut self, connection: RevelArc<Box<NetworkConnectionToClient>>) {
        self.parent.on_server_ready_default(connection)
    }

    /// 当客户端添加使用 NetworkClient.AddPlayer 的新播放器时，请在服务器上调用
    fn on_server_add_player(&mut self, connection: RevelArc<Box<NetworkConnectionToClient>>) {
        self.client_index += 1;

        if let Some(mut world) = WorldManager::active_world().upgrade() {
            if world.get_scene_path() == self.room_scene {
                self.set_all_players_ready(false);

                let new_room_game_object = self
                    .on_room_server_create_room_player(connection.clone())
                    .map_or_else(
                        || {
                            if let Some(prefab) = Metadata::get_prefab(&*self.room_player_prefab) {
                                return Some(GameObject::instantiate(prefab));
                            }
                            None
                        },
                        |f| Some(f),
                    );

                // println!("{}", new_room_game_object.clone().unwrap().name);

                NetworkServer::add_player_for_connection(connection, new_room_game_object.unwrap());
            } else {
                log::info!("Not in Room scene...disconnecting");
                connection.disconnect.call(());
            }
        }
    }

    fn on_server_change_scene(&mut self, scene_name: String) {
        for room_player in self.room_slots.iter() {
            if let Some(mut room_player) = room_player.upgrade() {
                if let Some(identity) = room_player.network_identity.upgrade() {
                    if NetworkServer.active {
                        room_player.set_ready_to_begin(false);

                        if let (Some(identity_connection), Some(room_player_game_object)) = (
                            identity.connection().upgrade(),
                            room_player.game_object.upgrade(),
                        ) {
                            NetworkServer::replace_player_for_connection(
                                identity_connection,
                                room_player_game_object,
                                ReplacePlayerOptions::KeepActive,
                            );
                        }
                    }
                }
            }
        }

        self.set_all_players_ready(false);

        self.parent.on_server_change_scene_default(scene_name);
    }

    fn on_server_scene_changed(&mut self, scene_name: String) {
        if scene_name != self.room_scene {
            for pending_player in self.pending_players.clone().iter() {
                if let (Some(connection), Some(room_player)) = (
                    pending_player.connection.upgrade(),
                    pending_player.room_player.upgrade(),
                ) {
                    self.scene_loaded_for_player(connection, room_player)
                }
            }
            self.pending_players.clear();
        }

        self.on_room_server_scene_changed.call((scene_name,));
    }

    fn on_start_server(&mut self) {
        if self.room_scene.is_empty() {
            log::error!("NetworkRoomManager RoomScene is empty. Set the RoomScene in the inspector for the NetworkRoomManager");
            return;
        }

        if self.gameplay_scene.is_empty() {
            log::error!("NetworkRoomManager PlayScene is empty. Set the PlayScene in the inspector for the NetworkRoomManager");
            return;
        }

        self.on_room_start_server.call(())
    }
    fn on_stop_server(&mut self) {
        self.room_slots.clear();
        self.on_room_stop_server.call(());
    }
}

impl NetworkRoomManager {
    pub fn recalculate_room_player_indices(&self) {
        if self.room_slots.len() > 0 {
            let mut index = 0;
            for room_player in self.room_slots.iter() {
                if let Some(mut room_player) = room_player.upgrade() {
                    room_player.set_index(index);
                }
                index += 1;
            }
        }
    }
}

impl NetworkRoomManager {
    fn scene_loaded_for_player(
        &mut self,
        connection: RevelArc<Box<NetworkConnectionToClient>>,
        room_player: RevelArc<GameObject>,
    ) {
        if WorldManager::active_world().get().unwrap().get_scene_path() == self.room_scene {
            let player = PendingPlayer {
                connection: connection.downgrade(),
                room_player: room_player.downgrade(),
            };
            self.pending_players.push(player);
        }

        let mut game_player =
            self.on_room_server_create_game_player(connection.clone(), room_player.clone());
        if game_player.is_none() {
            if let Some(prefab) = Metadata::get_prefab(&self.room_player_prefab) {
                let mut game_object = GameObject::instantiate(prefab);
                if let Some(start_position) = self.get_start_position() {
                    game_object.transform.position = start_position.position;
                    game_object.transform.rotation = start_position.rotation;
                }
                game_player = Some(game_object);
            } else {
                log::error!(
                    "NetworkRoomManager: Failed to find room player prefab: {}",
                    self.room_player_prefab
                );
                return;
            }
        }

        if !self.on_room_server_scene_loaded_for_player(
            connection.clone(),
            room_player,
            game_player.clone().unwrap(),
        ) {
            return;
        }

        NetworkServer::replace_player_for_connection(
            connection,
            game_player.clone().unwrap(),
            ReplacePlayerOptions::KeepActive,
        );
    }

    fn call_on_client_enter_room(&self) {
        self.on_room_client_enter.call(());
        for room_slot in self.room_slots.iter() {
            if let Some(room_slot) = room_slot.upgrade() {
                room_slot.on_client_enter_room.call(());
            }
        }
    }

    #[action]
    fn on_room_server_create_room_player(
        &self,
        conn: RevelArc<Box<NetworkConnectionToClient>>,
    ) -> Option<RevelArc<GameObject>> {
        None
    }
    #[action]
    fn on_room_server_create_game_player(
        &self,
        conn: RevelArc<Box<NetworkConnectionToClient>>,
        room_player: RevelArc<GameObject>,
    ) -> Option<RevelArc<GameObject>> {
        None
    }

    /// 这允许自定义服务器上的游戏玩家对象的创建。
    #[action]
    fn on_room_server_add_player(&mut self, conn: RevelArc<Box<NetworkConnectionToClient>>) {
        self.parent.on_server_add_player(conn)
    }
    #[action]
    pub fn on_room_server_scene_loaded_for_player(
        &self,
        conn: RevelArc<Box<NetworkConnectionToClient>>,
        room_player: RevelArc<GameObject>,
        game_player: RevelArc<GameObject>,
    ) -> bool {
        true
    }

    /// 这是从 NetworkRoomPlayer.CmdChangeReadyState 上的服务器上调用的，当客户端指示“就绪状态更改”时。
    #[action]
    pub fn ready_status_changed(&mut self) {
        println!("pub fn ready_status_changed(&mut self)");
        let mut current_players = 0;
        let mut ready_players = 0;

        for player in self.room_slots.iter() {
            if let Some(player) = player.upgrade() {
                current_players += 1;
                if *player.get_ready_to_begin() {
                    ready_players += 1;
                }
            }
        }

        if current_players == ready_players {
            self.check_ready_to_begin();
        } else {
            self.set_all_players_ready(false);
        }
    }

    /// <summary>当房间中的所有玩家准备就绪时，这是在服务器上调用的。</summary>
    #[action]
    pub fn on_room_server_players_ready(&mut self) {
        //所有玩家都准备开始，开始游戏
        let gameplay_scene = self.gameplay_scene.clone();
        self.server_change_scene(&gameplay_scene);
    }
}

impl NetworkRoomManager {
    fn on_server_error(
        &mut self,
        connection: RevelArc<Box<NetworkConnectionToClient>>,
        error: TransportError,
        reason: String,
    ) {
    }

    fn on_server_transport_exception(
        &mut self,
        connection: RevelArc<Box<NetworkConnectionToClient>>,
        error: Box<dyn Error>,
    ) {
    }
}

impl MonoBehaviour for NetworkRoomManager {
    fn awake(&mut self) {
        self.parent.awake();
    }

    fn start(&mut self) {
        self.parent.start();
    }
    fn update(&mut self) {
        self.parent.update();
    }

    fn late_update(&mut self) {
        self.parent.late_update();
    }
}

impl NetworkRoomManagerInitialize for NetworkRoomManager {
    fn initialize(&mut self, metadata: &MetadataNetworkManagerWrapper) {
        let config = metadata.get::<MetadataNetworkRootManager>();
        self.min_players = config.min_players;
        self.room_player_prefab = config.room_player_prefab.asset_path.clone();
        self.room_slots = Default::default();
        self.room_scene = config.room_scene.clone();
        self.gameplay_scene = config.gameplay_scene.clone();
        self.client_index = config.client_index;

        self.on_client_scene_changed =
            SelfMutAction::new(self.weak.clone(), Self::on_client_scene_changed);
        self.on_start_server = SelfMutAction::new(self.weak.clone(), Self::on_start_server);
        self.on_stop_server = SelfMutAction::new(self.weak.clone(), Self::on_stop_server);
        self.on_server_connect = SelfMutAction::new(self.weak.clone(), Self::on_server_connect);
        self.on_server_change_scene =
            SelfMutAction::new(self.weak.clone(), Self::on_server_change_scene);
        self.on_server_scene_changed =
            SelfMutAction::new(self.weak.clone(), Self::on_server_scene_changed);
        self.on_server_disconnect =
            SelfMutAction::new(self.weak.clone(), Self::on_server_disconnect);
        self.on_server_error = SelfMutAction::new(self.weak.clone(), Self::on_server_error);
        self.on_server_transport_exception =
            SelfMutAction::new(self.weak.clone(), Self::on_server_transport_exception);

        self.on_server_add_player =
            SelfMutAction::new(self.weak.clone(), Self::on_server_add_player);
    }
}

impl NetworkRoomManager {
    fn on_client_scene_changed(&mut self) {
        // let name = std::any::type_name::<Self>();
        // println!("{}", name.split("::").last().unwrap_or_default());
        // println!("NetworkManager: Client scene changed 111");
        // 这里可以添加更多的逻辑处理
    }
    //
    // pub fn ready_status_changed(&mut self) {
    //     let mut current_players = 0;
    //     let mut ready_players = 0;
    //
    //     for item in &self.room_slots {
    //         current_players += 1;
    //         if *item.get_ready_to_begin() {
    //             ready_players += 1;
    //         }
    //     }
    //
    //     if current_players == self.min_players {
    //         self.check_ready_to_begin();
    //     } else {
    //         self.all_players_ready = false;
    //     }
    // }

    fn check_ready_to_begin(&mut self) {
        if let Some(world) = WorldManager::active_world().upgrade() {
            if world.get_scene_path() != self.get_network_scene_name() {
                return;
            }
        }

        let number_of_ready_players = NetworkServer
            .connections
            .iter()
            .filter(|(a, conn)| {
                if let Some(identity) = conn.identity.upgrade() {
                    if let Some(game_object) = identity.game_object.upgrade() {
                        if let Some(player) = game_object.try_get_component2::<NetworkRoomPlayer>()
                        {
                            return *player.get_ready_to_begin();
                        }
                    }
                }
                false
            })
            .count();

        let enough_ready_players =
            self.min_players <= 0 || number_of_ready_players >= self.min_players as usize;

        if enough_ready_players {
            self.pending_players.clear();
            self.set_all_players_ready(true);
        } else {
            self.set_all_players_ready(false);
        }
    }
}
