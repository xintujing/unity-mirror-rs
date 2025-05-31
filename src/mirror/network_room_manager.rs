use crate::commons::action::SelfMutAction;
use crate::metadata_settings::mirror::metadata_network_manager::MetadataNetworkManagerWrapper;
use crate::mirror::{NetworkConnection, NetworkManager};
use crate::mirror::NetworkManagerCallbacks;
use crate::unity_engine::MonoBehaviour;
use std::any::Any;
use std::error::Error;
use unity_mirror_macro::{namespace, network_manager, NetworkManagerFactory};
use crate::commons::revel_arc::RevelArc;
use crate::mirror::transport::TransportError;

#[network_manager(parent(NetworkManager, callbacks = NetworkManagerCallbacks))]
#[namespace(prefix = "Mirror")]
#[derive(NetworkManagerFactory)]
pub struct NetworkRoomManager {}
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
    fn update(&mut self) {
        if let Some(parent) = self.parent.get() {
            parent.update();
        }
        // println!("Mirror: NetworkRoomManager update");
    }
}

impl NetworkRoomManagerInitialize for NetworkRoomManager {
    fn initialize(&mut self, metadata: &MetadataNetworkManagerWrapper) {
        // let weak = self.weak.clone();
        // self.on_client_scene_changed = Some(ActionWrapper::new(move || {
        //     if let Some(this) = weak.upgrade() {
        //         Self::on_client_scene_changed(unsafe { &mut **this.get() }, );
        //     }
        // }));
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
