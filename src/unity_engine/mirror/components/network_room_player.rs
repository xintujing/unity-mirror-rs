use crate::unity_engine::mirror::network_behaviour_trait::{
    NetworkBehaviour, NetworkBehaviourDeserializer, NetworkBehaviourSerializer,
};
use crate::unity_engine::mono_behaviour::MonoBehaviour;
use unity_mirror_macro::namespace;

// #[network_behaviour]
#[namespace("Mirror")]
pub struct NetworkRoomPlayer {
    // #[sync_var]
    ready_to_begin: bool,
    // #[sync_var]
    index: i32,
}

impl NetworkRoomPlayer {
    // #[command]
    pub fn cmd_change_ready_state(&self, ready_state: bool) {
        // self.net_id
    }
}

// impl NetworkRoomPlayerHooks for NetworkRoomPlayer {
//     fn ready_to_begin_changed(&mut self, old_value: &bool, new_value: &bool) {
//         todo!()
//     }
//
//     fn index_changed(&mut self, old_value: &i32, new_value: &i32) {
//         todo!()
//     }
// }

impl MonoBehaviour for NetworkRoomPlayer {
    fn start(&mut self) {}

    fn on_disable(&mut self) {
        println!("NetworkRoomPlayer: on_disable");
    }
}

impl NetworkBehaviourSerializer for NetworkRoomPlayer {
    fn serialize(&self) {
        // let x = self.game_object().upgrade().unwrap().get();
    }
}

impl NetworkBehaviourDeserializer for NetworkRoomPlayer {
    fn deserialize(&self) {}
}

impl NetworkBehaviour for NetworkRoomPlayer {}
