use crate::commons::action::SelfMutAction;
use crate::commons::object::Object;
use crate::commons::revel_arc::RevelArc;
use crate::commons::revel_weak::RevelWeak;
use crate::metadata_settings::mirror::network_behaviours::metadata_network_behaviour::MetadataNetworkBehaviourWrapper;
use crate::metadata_settings::mirror::network_behaviours::metadata_network_room_player::MetadataNetworkRoomPlayer;
use crate::mirror::transport::TransportChannel;
use crate::mirror::TNetworkBehaviour;
use crate::mirror::{NetworkBehaviour, NetworkManager, NetworkRoomManager};
use crate::unity_engine::GameObject;
use crate::unity_engine::MonoBehaviour;
use std::any::TypeId;
use unity_mirror_macro_rs::{command, namespace, network_behaviour, target_rpc};

#[namespace(prefix = "Mirror")]
#[network_behaviour(parent(NetworkBehaviour), metadata(MetadataNetworkRoomPlayer))]
pub struct NetworkRoomPlayer {
    #[sync_variable]
    ready_to_begin: bool,
    #[sync_variable]
    index: i32,

    pub on_client_enter_room: SelfMutAction<(), ()>,
}

impl NetworkRoomPlayerOnChangeCallback for NetworkRoomPlayer {}

impl NetworkRoomPlayer {
    #[command(NetworkRoomPlayer, authority)]
    pub fn cmd_change_ready_state(&mut self, ready_state: bool) {
        println!("pub fn cmd_change_ready_state(&mut self, ready_state: bool)");
        self.set_ready_to_begin(ready_state);
        NetworkManager::singleton::<NetworkRoomManager>(|room| {
            room.ready_status_changed();
        });
    }
}

impl MonoBehaviour for NetworkRoomPlayer {
    fn start(&mut self) {}

    fn on_disable(&mut self) {
        println!("NetworkRoomPlayer: on_disable");
    }
}

impl TNetworkBehaviour for NetworkRoomPlayer {
    fn new(
        weak_game_object: RevelWeak<GameObject>,
        metadata: &MetadataNetworkBehaviourWrapper,
    ) -> Self
    where
        Self: Sized,
    {
        Self::default()
    }
}
