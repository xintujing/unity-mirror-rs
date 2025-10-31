use crate::business::network_syc::game_room_list_manger::GameRoomListManger;
use crate::business::network_syc::network_room_player_ext::NetworkRoomPlayerExt;
use crate::metadata_settings::mirror::metadata_network_behaviour::MetadataNetworkBehaviourWrapper;
use crate::mirror::component::{Component, NetworkBehaviour, UnityEngineLifespan};
use crate::mirror::components::c_sync_var::CSyncVar;
use crate::mirror::identity::Identity;
use crate::mirror::pointer::WeakMutex;
use dda_macro::{command, init_state, namespace, rpc_impl, Component, ComponentState};
use nalgebra::Vector3;

#[derive(ComponentState)]
pub struct GamePlayerState {
    pub is_all_players_loading_status: CSyncVar<bool>,
    pub keep_poker: CSyncVar<i32>,
    pub is_shrink: CSyncVar<bool>,
    pub is_cut_the_card: CSyncVar<bool>,
    pub is_countdown: CSyncVar<bool>,
    pub room_player_net_id: CSyncVar<u32>,

    pub room_player: Option<NetworkRoomPlayerExt>,
}

#[derive(Component, Clone)]
#[namespace(prefix = "HotUpdate.PlayAlgorithmSyc")]
pub struct GamePlayer {
    id: String,
    #[parent]
    network_behaviour: NetworkBehaviour,
}

impl UnityEngineLifespan for GamePlayer {}

impl Component for GamePlayer {
    fn new(behaviour_settings: &MetadataNetworkBehaviourWrapper, weak_mutex_identity: WeakMutex<Identity>, index: u8) -> (Self, usize, usize)
    where
        Self: Sized,
    {
        let (behaviour, sync_obj_index, sync_var_index) = NetworkBehaviour::new(behaviour_settings, weak_mutex_identity, index);
        let (id, sync_obj_index, sync_var_index) = init_state!(
            Some(behaviour.id.clone()),
            (sync_obj_index, sync_var_index),
            GamePlayerState {
                is_all_players_loading_status: CSyncVar::new(Default::default()),
                keep_poker: CSyncVar::new(0),
                is_shrink: CSyncVar::new(Default::default()),
                is_cut_the_card: CSyncVar::new(Default::default()),
                is_countdown: CSyncVar::new(Default::default()),
                room_player_net_id: CSyncVar::new(Default::default()),
                room_player: None,
            }
        );

        (
            Self {
                id: id.clone().unwrap(),
                network_behaviour: behaviour,
            },
            sync_obj_index,
            sync_var_index,
        )
    }

    fn on_start_server(&self) {
        if let Some(grlms) = GameRoomListManger::singleton() {
            grlms.game_players.push(self.clone());
        }
    }

    fn on_stop_server(&self) {
        if let Some(grlms) = GameRoomListManger::singleton() {
            grlms.game_players.retain(|player| player.id != self.id);
        }
    }
}

#[rpc_impl]
impl GamePlayer {
    #[command]
    fn cmd_start_shuffle_card(&self, _fun_board_shuffle_position: Vector3<f32>, _fun_board_shuffle_rotation: Vector3<f32>) {
        log::debug!("cmd_start_shuffle_card");
    }

    #[command]
    fn cmd_next_player_start_cut_cards_ani(&self) {
        log::debug!("cmd_next_player_start_cut_cards_ani");
    }

    #[command]
    fn cmd_start_cut_card_tracking(&self, _next_player_index: i32) {
        log::debug!("cmd_start_cut_card_tracking");
    }

    #[command]
    fn cmd_is_game_scene_load_complete_changed(&self) {
        log::debug!("cmd_is_game_scene_load_complete_changed");
    }
}
