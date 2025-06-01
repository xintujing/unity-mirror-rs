use crate::commons::action::SelfMutAction;
use crate::metadata_settings::mirror::metadata_network_manager::MetadataNetworkManagerWrapper;
use crate::mirror::{NetworkConnection, NetworkIdentity, NetworkManager};
use crate::mirror::NetworkManagerCallbacks;
use crate::unity_engine::MonoBehaviour;
use std::any::Any;
use std::collections::HashSet;
use std::error::Error;
use unity_mirror_macro::{namespace, network_manager, NetworkManagerFactory};
use crate::commons::revel_arc::RevelArc;
use crate::commons::revel_weak::RevelWeak;
use crate::metadata_settings::mirror::metadata_network_root_manager::MetadataNetworkRootManager;
use crate::mirror::components::network_room_player::NetworkRoomPlayer;
use crate::mirror::transport::TransportError;

pub struct PendingPlayer {
    pub connection: RevelWeak<NetworkConnection>,
    pub room_player: RevelWeak<NetworkIdentity>,
}

#[allow(unused)]
#[network_manager(parent(NetworkManager, callbacks = NetworkManagerCallbacks))]
#[namespace(prefix = "Mirror")]
#[derive(NetworkManagerFactory)]
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
//
// impl crate::commons::action::Arguments for &NetworkRoomManager {}
// impl FromArguments for NetworkRoomManager {
//     fn to_args(&self) -> Self {
//         self
//     }
// }

impl NetworkManagerCallbacks for NetworkRoomManager {
    fn on_start_server(&mut self) {
        // self.qwer()
    }

    fn on_stop_server(&mut self) {}

    fn on_server_connect(&mut self, connection: RevelArc<NetworkConnection>) {}

    fn on_server_change_scene(&mut self, scene_name: String) {}

    fn on_server_scene_changed(&mut self, scene_name: String) {}

    fn on_server_disconnect(&self, connection: RevelArc<NetworkConnection>) {}

    fn on_server_error(&self, connection: RevelArc<NetworkConnection>, error: TransportError, reason: String) {}

    fn on_server_transport_exception(&self, connection: RevelArc<NetworkConnection>, error: Box<dyn Error>) {}
}

impl MonoBehaviour for NetworkRoomManager {
    fn awake(&mut self) {
        self.on_client_scene_changed =
            SelfMutAction::new(self.weak.clone(), Self::on_client_scene_changed);

        if let Some(parent) = self.parent.get() {
            parent.awake();
            // if let Some(game_object) = self.game_object.get() {
            //     let option = game_object.find_component(self).unwrap();
            //     let instance = unsafe {
            //         &*(&option as *const dyn Any
            //             as *const RevelWeak<Box<dyn NetworkManagerCallbacks>>)
            //     };
            //     parent.set_callbacks(instance.clone());
            // }
        }
        // println!("NetworkRoomManager awake");
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
        // println!("Mirror: NetworkRoomManager update");
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
    }
}

impl NetworkRoomManager {
    pub fn qwer(&mut self, i: i32) -> i32 {
        // self.qwe();
        println!("NetworkRoomManager qwer {}", i);
        77
    }

    fn on_client_scene_changed(&mut self) {
        // let name = std::any::type_name::<Self>();
        // println!("{}", name.split("::").last().unwrap_or_default());
        // println!("NetworkManager: Client scene changed 111");
        // 这里可以添加更多的逻辑处理
    }
}
