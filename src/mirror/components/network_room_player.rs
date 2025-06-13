use crate::commons::action::SelfMutAction;
use crate::macro_namespace::*;
use crate::macro_network_behaviour::*;
use crate::metadata_settings::MetadataNetworkBehaviourWrapper;
use crate::metadata_settings::MetadataNetworkRoomPlayer;
use crate::mirror::{NetworkManager, NetworkRoomManager, NetworkServer, TNetworkBehaviour};
use crate::unity_engine::MonoBehaviour;
use crate::unity_engine::{GameObject, WorldManager};
use std::hash::{Hash, Hasher};

#[namespace(prefix = "Mirror")]
#[network_behaviour(parent(NetworkBehaviour), metadata(MetadataNetworkRoomPlayer))]
pub struct NetworkRoomPlayer {
    #[sync_var]
    ready_to_begin: bool,
    #[sync_var]
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
    #[command(NetworkRoomPlayer)]
    pub fn cmd_change_ready_state(&mut self, ready_state: bool) {
        self.set_ready_to_begin(ready_state);
        log::debug!("My index: {}, ready state: {}", self.get_index(), self.get_ready_to_begin());
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

    fn on_disable(&mut self) {}
}

impl TNetworkBehaviour for NetworkRoomPlayer {
    fn new(
        _weak_game_object: RevelWeak<GameObject>,
        _metadata: &MetadataNetworkBehaviourWrapper,
    ) -> Self
    where
        Self: Sized,
    {
        Self::default()
    }
}
