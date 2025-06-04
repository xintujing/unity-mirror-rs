use crate::commons::action::SelfMutAction;
use crate::commons::revel_arc::RevelArc;
use crate::commons::revel_weak::RevelWeak;
use crate::metadata_settings::mirror::metadata_network_manager::MetadataNetworkManagerWrapper;
use crate::metadata_settings::mirror::metadata_network_root_manager::MetadataNetworkRootManager;
use crate::mirror::components::network_room_player::NetworkRoomPlayer;
use crate::mirror::transport::TransportError;
use crate::mirror::NetworkManagerVirtualTrait;
use crate::mirror::{NetworkConnection, NetworkIdentity, NetworkManager};
use crate::unity_engine::MonoBehaviour;
use std::any::Any;
use std::collections::HashSet;
use std::error::Error;
use unity_mirror_macro::{namespace, network_manager, NetworkManagerFactory};

pub struct PendingPlayer {
    pub connection: RevelWeak<NetworkConnection>,
    pub room_player: RevelWeak<NetworkIdentity>,
}

#[namespace(prefix = "Mirror")]
#[network_manager(parent(NetworkManager, callbacks = NetworkManagerVirtualTrait))]
pub struct NetworkRoomManager {
    // 最少可以自动启动游戏的玩家数量
    pub min_players: i32,
    // 预制可用于房间播放器
    pub room_player_prefab: String,
    pub room_slots: HashSet<NetworkRoomPlayer>,
    // 用于房间的场景 这类似于 NetworkRoomManager 的 offline scene
    pub room_scene: String,
    // 从房间玩游戏的场景 这类似于 NetworkRoomManager 的online scene。
    pub gameplay_scene: String,
    // 房间里的玩家名单
    pub pending_players: Vec<PendingPlayer>,
    // 诊断标志 表明所有玩家都准备好玩
    all_players_ready: bool,
    client_index: usize,
}

impl NetworkManagerVirtualTrait for NetworkRoomManager {
    fn on_start_server(&mut self) {}

    fn on_stop_server(&mut self) {}

    fn on_server_connect(&mut self, connection: RevelArc<NetworkConnection>) {}

    fn on_server_change_scene(&mut self, scene_name: String) {}

    fn on_server_scene_changed(&mut self, scene_name: String) {}

    fn on_server_disconnect(&self, connection: RevelArc<NetworkConnection>) {}

    fn on_server_error(
        &self,
        connection: RevelArc<NetworkConnection>,
        error: TransportError,
        reason: String,
    ) {
    }

    fn on_server_transport_exception(
        &self,
        connection: RevelArc<NetworkConnection>,
        error: Box<dyn Error>,
    ) {
    }
}

impl MonoBehaviour for NetworkRoomManager {
    fn awake(&mut self) {
        if let Some(parent) = self.parent.get() {
            parent.awake();
        }
    }

    fn start(&mut self) {
        if let Some(parent) = self.parent.get() {
            parent.start();
        }
    }
    fn update(&mut self) {
        if let Some(parent) = self.parent.get() {
            parent.update();
        }
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
    }
}

impl NetworkRoomManager {
    fn on_client_scene_changed(&mut self) {
        // let name = std::any::type_name::<Self>();
        // println!("{}", name.split("::").last().unwrap_or_default());
        // println!("NetworkManager: Client scene changed 111");
        // 这里可以添加更多的逻辑处理
    }

    pub fn ready_status_changed(&mut self) {
        let mut current_players = 0;
        let mut ready_players = 0;

        for item in &self.room_slots {
            current_players += 1;
            if item.is_ready() {
                ready_players += 1;
            }
        }

        if current_players == self.min_players {
            self.check_ready_to_begin();
        } else {
            self.all_players_ready = false;
        }
    }

    fn check_ready_to_begin(&mut self) {
        // TODO
    }
}
