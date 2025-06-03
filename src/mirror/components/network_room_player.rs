use crate::commons::object::Object;
use crate::commons::revel_arc::RevelArc;
use crate::commons::revel_weak::RevelWeak;
use crate::metadata_settings::mirror::network_behaviours::metadata_network_behaviour::MetadataNetworkBehaviourWrapper;
use crate::metadata_settings::mirror::network_behaviours::metadata_network_room_player::MetadataNetworkRoomPlayer;
use crate::mirror::transport::TransportChannel;
use crate::mirror::NetworkBehaviour;
use crate::mirror::TNetworkBehaviour;
use crate::unity_engine::GameObject;
use crate::unity_engine::MonoBehaviour;
use std::any::TypeId;
use unity_mirror_macro::{command, namespace, network_behaviour, target_rpc};

#[allow(unused)]
#[network_behaviour(parent(NetworkBehaviour), metadata(MetadataNetworkRoomPlayer))]
#[namespace(prefix = "Mirror")]
pub struct NetworkRoomPlayer {
    // #[sync_var]
    ready_to_begin: bool,
    // #[sync_var]
    index: i32,
}

// #[rpc_impl]
impl NetworkRoomPlayer {
    #[command(NetworkRoomPlayer, authority)]
    pub fn cmd_change_ready_state(&mut self, ready_state: bool) {
        // // Self::COMPONENT_NAME;
        // let x = stringify!(NetworkRoomPlayer);
        // let name = std::any::type_name::<Self>();
        // println!("{}", name);
        // // self.net_id
    }
}

impl MonoBehaviour for NetworkRoomPlayer {
    fn start(&mut self) {}

    fn on_disable(&mut self) {
        println!("NetworkRoomPlayer: on_disable");
    }
}

#[allow(unused)]
impl NetworkRoomPlayer {
    fn instance(
        weak_game_object: RevelWeak<GameObject>,
        metadata: &MetadataNetworkBehaviourWrapper,
    ) -> (
        Vec<(RevelArc<Box<dyn MonoBehaviour>>, TypeId)>,
        RevelWeak<crate::mirror::NetworkBehaviour>,
        u8,
        u8,
    )
    where
        Self: Sized,
    {
        todo!()
    }
}

impl TNetworkBehaviour for NetworkRoomPlayer {
    fn new(
        weak_game_object: &RevelWeak<GameObject>,
        metadata: &MetadataNetworkBehaviourWrapper,
    ) -> Self
    where
        Self: Sized,
    {
        Self::default()
    }
}
impl NetworkRoomPlayerOnChangeCallback for NetworkRoomPlayer {}
