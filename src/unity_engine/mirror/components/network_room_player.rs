use crate::commons::revel_arc::RevelArc;
use crate::commons::revel_weak::RevelWeak;
use crate::metadata_settings::mirror::network_behaviours::metadata_network_behaviour::MetadataNetworkBehaviourWrapper;
use crate::unity_engine::mirror::network_behaviour_trait::{
    NetworkBehaviour, NetworkBehaviourInstance
    ,
};
use crate::unity_engine::mono_behaviour::MonoBehaviour;
use crate::unity_engine::GameObject;
use std::any::TypeId;
use unity_mirror_macro::namespace;

// #[network_behaviour]
#[namespace(prefix = "Mirror")]
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

impl NetworkBehaviourInstance for NetworkRoomPlayer {
    fn instance(
        weak_game_object: RevelWeak<GameObject>,
        metadata: &MetadataNetworkBehaviourWrapper,
    ) -> (
        Vec<(RevelArc<Box<dyn MonoBehaviour>>, TypeId)>,
        RevelWeak<crate::unity_engine::mirror::NetworkBehaviour>,
        u8,
        u8,
    )
    where
        Self: Sized,
    {
        todo!()
    }
}
