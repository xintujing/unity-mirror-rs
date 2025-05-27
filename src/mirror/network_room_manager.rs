use crate::commons::revel_arc::RevelArc;
use crate::commons::revel_weak::RevelWeak;
use crate::metadata_settings::mirror::metadata_network_manager::MetadataNetworkManagerWrapper;
use crate::mirror::network_manager::NetworkManager;
use crate::mirror::NetworkManagerCallbacks;
use crate::unity_engine::MonoBehaviour;
use std::any::Any;
use std::cell::UnsafeCell;
use std::mem;
use std::ops::Deref;
use std::sync::{Arc, Weak};
use unity_mirror_macro::{callbacks, namespace, network_manager, NetworkManagerFactory};

#[network_manager(parent(NetworkManager, callbacks = NetworkManagerCallbacks))]
#[namespace(prefix = "Mirror")]
#[derive(NetworkManagerFactory)]
pub struct NetworkRoomManager {}

impl NetworkManagerCallbacks for NetworkRoomManager {
    fn on_start_server(&mut self) {
        self.qwer()
    }

    fn on_stop_server(&mut self) {
        todo!()
    }
}

impl MonoBehaviour for NetworkRoomManager {
    fn awake(&mut self) {
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
        println!("NetworkRoomManager awake");
    }
    fn update(&mut self) {
        if let Some(parent) = self.parent.get() {
            parent.update();
        }
        println!("Mirror: NetworkRoomManager update");
    }
}

impl NetworkRoomManagerInitialize for NetworkRoomManager {
    fn initialize(&mut self, metadata: &MetadataNetworkManagerWrapper) {}
}

impl NetworkRoomManager {
    pub fn qwer(&self) {
        // self.qwe();
        println!("NetworkRoomManager qwer");
    }
}
