use crate::commons::action::SelfMutAction;
use crate::commons::object::Object;
use crate::commons::revel_arc::RevelArc;
use crate::commons::revel_weak::RevelWeak;
use crate::metadata_settings::mirror::network_behaviours::metadata_network_behaviour::MetadataNetworkBehaviourWrapper;
use crate::metadata_settings::mirror::network_behaviours::metadata_network_room_player::MetadataNetworkRoomPlayer;
use crate::mirror::transport::TransportChannel;
use crate::mirror::{NetworkBehaviour, NetworkManager, NetworkRoomManager, NetworkWriter, WriteCompress};
use crate::mirror::{NetworkServer, TNetworkBehaviour};
use crate::unity_engine::MonoBehaviour;
use crate::unity_engine::{GameObject, WorldManager};
use std::any::TypeId;
use std::hash::{Hash, Hasher};
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

impl Eq for NetworkRoomPlayer {}

impl PartialEq for NetworkRoomPlayer {
    fn eq(&self, other: &Self) -> bool {
        self.weak == other.weak
    }
}

impl Hash for NetworkRoomPlayer {
    // TODO 完善
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(unsafe { *(self.weak.as_ptr() as *const u64) });
    }
}


impl NetworkRoomPlayerOnChangeCallback for NetworkRoomPlayer {}

impl NetworkRoomPlayer {
    #[command(NetworkRoomPlayer, authority)]
    pub fn cmd_change_ready_state(&mut self, ready_state: bool) {
        self.set_ready_to_begin(ready_state);
        println!("My index: {}, ready state: {}", self.get_index(), self.get_ready_to_begin());
        NetworkManager::singleton::<NetworkRoomManager, _>(|room| {
            // TODO: 这里需要处理一下，可能会有问题
            room.ready_status_changed();
        });
    }
}

impl MonoBehaviour for NetworkRoomPlayer {
    fn start(&mut self) {
        NetworkManager::singleton::<NetworkRoomManager, _>(|room| {
            if let Some(game_object) = self.game_object.upgrade() {
                WorldManager::dont_destroy_object(game_object);
            }

            room.room_slots.insert(self.weak.clone());

            if NetworkServer.active {
                room.recalculate_room_player_indices();
            }
        })
    }

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
