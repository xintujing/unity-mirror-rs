use crate::metadata_settings::metadata_player_ext::MetadataPlayerExt;
use crate::scripts::common::Common;
use crate::scripts::network_manager_ext::{NetworkManagerExt, SeatLocation};
use crate::scripts::network_manager_ext_status::StageStatus;
use crate::scripts::poker_controller::PokerController;
use unity_mirror_rs::macro_namespace::*;
use unity_mirror_rs::macro_network_behaviour::*;
use unity_mirror_rs::mirror::{NetworkManager, NetworkServer, SyncList};
use unity_mirror_rs::unity_engine::WorldManager;
use unity_mirror_rs::{data_type_deserialize, data_type_serialize};

#[namespace(prefix = "DaDaA.Scripts")]
#[derive(Default, Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
pub enum PlayerIdentity {
    #[default]
    Ordinary = 0,       // 普通
    DoubleInverse,      // 双反
    DoubleBright,       // 双明
    SingleBrightDark,   // 单明暗
    SingleBright,       // 单明
    SingleBrightQuilt,  // 单明被杵
    SingleDark,         // 单暗
    SingleDarkPestle,   // 单暗杵
    SingleDark2Bright,  // 单暗变明
}

data_type_serialize!(
    (
       PlayerIdentity
    ),
    |value, writer| writer.write_blittable(*value)
);
data_type_deserialize!(
    (
        PlayerIdentity
    ),
    {|reader| reader.read_blittable()}
);

#[namespace(prefix = "DaDaA.Scripts")]
#[network_behaviour(parent(NetworkBehaviour), metadata(MetadataPlayerExt))]
pub struct PlayerExt {
    #[sync_var]
    nme_index: u8,
    #[sync_var]
    ready_to_begin: bool,
    #[sync_var]
    room_owner: bool,
    #[sync_var]
    player_stage_status: StageStatus,
    #[sync_var]
    player_ext_identity: PlayerIdentity,
    #[sync_var]
    countdown: f64,
    #[sync_var]
    integral: u64,

    countdown_seconds: f64,
    countdown_instant: Option<std::time::Instant>,
}

impl MonoBehaviour for PlayerExt {
    fn start(&mut self) {
        if let Some(nme) = NetworkManager::singleton_mut::<NetworkManagerExt>() {
            if nme.dont_destroy_on_load {
                if let Some(game_object) = self.game_object.upgrade() {
                    WorldManager::dont_destroy_object(game_object);
                }
            }

            nme.player_ext_s.push(self.weak.clone());

            if NetworkServer.active {
                nme.recalculate_room_player_indices();
            }
        }

        if let Some(nme) = NetworkManager::singleton_mut::<NetworkManagerExt>() {
            nme.seat_infos[*self.get_nme_index() as usize + 1].game_object = self.game_object.clone();
        }
    }

    fn update(&mut self) {

        // TODO: 暂时房主
        self.set_room_owner(*self.get_nme_index() == 0);

        // 更新倒计时
        if let Some(countdown_instant) = self.countdown_instant {
            let remaining = (self.countdown_seconds - countdown_instant.elapsed().as_secs_f64()).ceil();
            // 更新倒计时
            if remaining >= 0.0 {
                self.set_countdown(remaining);
            }
        }
    }

    fn on_disable(&mut self) {
        if let Some(nme) = NetworkManager::singleton_mut::<NetworkManagerExt>() {
            nme.player_ext_s.retain(|player_ext| {
                // 保留其它玩家
                !player_ext.ptr_eq(&self.weak)
            });

            if NetworkServer.active {
                nme.recalculate_room_player_indices();
            }
        }
    }
}

impl TNetworkBehaviour for PlayerExt {
    fn new(_weak_game_object: RevelWeak<GameObject>, _metadata: &MetadataNetworkBehaviourWrapper) -> Self
    where
        Self: Sized,
    {
        let player_ext = Self::default();

        player_ext
    }
}

impl PlayerExtOnChangeCallback for PlayerExt {
    fn on_nme_index_changed(&mut self, old_value: &u8, new_value: &u8) {
        if let Some(nme) = NetworkManager::singleton_mut::<NetworkManagerExt>() {
            // nme.seat_infos[*old_value as usize + 1].game_object = RevelWeak::default();
        }
    }
    fn on_ready_to_begin_changed(&mut self, _old_value: &bool, _new_value: &bool) {
        if let Some(nme) = NetworkManager::singleton_mut::<NetworkManagerExt>() {
            nme.ready_status_changed();
        }
    }

    fn on_room_owner_changed(&mut self, _old_value: &bool, new_value: &bool) {
        if *new_value {
            self.set_ready_to_begin(true);
            self.set_player_ext_identity(PlayerIdentity::SingleBright);
        }
    }

    fn on_player_stage_status_changed(&mut self, _old_value: &StageStatus, new_value: &StageStatus) {
        match new_value {
            StageStatus::None => {
                self.set_countdown(0.0);
            }
            StageStatus::Shuffle => {
                self.set_countdown(Common::SHUFFLE_COUNTDOWN);
            }
            StageStatus::Cut => {
                self.set_countdown(Common::CUT_COUNTDOWN);
            }
            StageStatus::Deal => {}
            StageStatus::BrightAce => {
                self.set_countdown(Common::BRIGHT_ACE_COUNTDOWN);
            }
            StageStatus::DoubleAceRev => {
                self.set_countdown(Common::BRIGHT_ACE_COUNTDOWN);
            }
            StageStatus::ChuAce => {
                self.set_countdown(Common::BRIGHT_ACE_COUNTDOWN);
            }
            StageStatus::PlayCards => {
                self.set_countdown(Common::PLAY_CARDS_COUNTDOWN);
            }
            StageStatus::WaitPlayCards => {
                self.set_countdown(Common::WAIT_PLAY_CARDS_COUNTDOWN);
            }
            StageStatus::Waiting => {
                self.set_countdown(0.0);
            }
            StageStatus::Settlement => {
                self.set_countdown(0.0);
            }
        }
    }

    fn on_countdown_changed(&mut self, old_value: &f64, new_value: &f64) {
        log::debug!("Player({:?}) - {}: old_value: {}, new_value: {}", self.get_player_stage_status(), self.get_nme_index() + 1,old_value, new_value);
        // 开始倒计时
        if *new_value >= 0.0 && *old_value <= 0.0 {
            self.countdown_instant = Some(std::time::Instant::now());
            self.countdown_seconds = *new_value;
        }
        // 倒计时结束
        if *new_value <= 0.0 && *old_value >= 0.0 {
            self.countdown_instant = None;
            self.countdown_seconds = 0.0;

            // 倒计时结束后，根据状态执行相应操作
            match self.get_player_stage_status() {
                StageStatus::None => {}
                StageStatus::Shuffle => {
                    // 不洗牌
                    if let Some(obj) = self.game_object.upgrade() {
                        if let Some(mut poker_controller) = obj.try_get_component2::<PokerController>() {
                            poker_controller.cmd_end_shuffle();
                        }
                    }
                }
                StageStatus::Cut => {
                    // 不切牌
                    if let Some(obj) = self.game_object.upgrade() {
                        if let Some(mut poker_controller) = obj.try_get_component2::<PokerController>() {
                            poker_controller.cmd_cut(0);
                        }
                    }
                }
                StageStatus::Deal => {}
                StageStatus::BrightAce | StageStatus::DoubleAceRev => {
                    log::debug!("PlayerExtStatus::BrightAce or PlayerExtStatus::DoubleAceRev");
                    // 不亮
                    if let Some(obj) = self.game_object.upgrade() {
                        if let Some(mut poker_controller) = obj.try_get_component2::<PokerController>() {
                            poker_controller.cmd_bright_a(&vec![]);
                        }
                    }
                }
                StageStatus::ChuAce => {
                    // 不杵
                    if let Some(obj) = self.game_object.upgrade() {
                        if let Some(mut poker_controller) = obj.try_get_component2::<PokerController>() {
                            poker_controller.cmd_pestle_a(false);
                        }
                    }
                }
                StageStatus::PlayCards => {
                    if let Some(nme) = NetworkManager::singleton_mut::<NetworkManagerExt>() {
                        match nme.last_pokers().unwrap().count() == 0 {
                            true => { // 没有上家出过牌，出牌
                                if let Some(obj) = self.game_object.upgrade() {
                                    if let Some(mut poker_controller) = obj.try_get_component2::<PokerController>() {
                                        if let Some(id) = poker_controller.poker_cards.iter().last().cloned() {
                                            poker_controller.cmd_play_cards(&[id]);
                                        }
                                    }
                                }
                            }
                            false => { // 有上家出过牌，不出
                                nme.next_player(self.game_object.clone(), StageStatus::None);
                            }
                        }
                    }
                }
                StageStatus::WaitPlayCards => {
                    // 更改状态为出牌
                    self.set_player_stage_status(StageStatus::PlayCards);
                }
                StageStatus::Waiting => {}
                StageStatus::Settlement => {}
            }
        }
    }
}

impl PlayerExt {
    fn get_seat_location<Func>(&self, index: u8, mut func: Func)
    where
        Func: FnMut(&mut SeatLocation),
    {
        if let Some(nme) = NetworkManager::singleton_mut::<NetworkManagerExt>() {
            if index == 0 || index > nme.seat_infos.len() as u8 {
                let seat_location = &mut nme.seat_infos[0];

                if let Some(game_object) = self.game_object.upgrade() {
                    seat_location.game_object = game_object.downgrade();
                    seat_location.transform = game_object.transform.clone().into_inner();
                }
                func(seat_location);
                return;
            }
            func(&mut nme.seat_infos[index as usize]);
        }
    }
}

// cmd rpc
impl PlayerExt {
    #[command(PlayerExt, requiresAuthority = false)]
    pub fn cmd_set_ready(&mut self) {
        // 更改状态
        self.set_ready_to_begin(!*self.get_ready_to_begin());
    }

    #[command(PlayerExt)]
    pub fn cmd_start_game(&mut self) {
        // 检查是否是房主
        if !self.get_room_owner() {
            log::error!("curr player are not the room owner, cannot start the game.");
            return;
        }

        // 开始游戏
        if let Some(nme) = NetworkManager::singleton_mut::<NetworkManagerExt>() {
            nme.start_game();
        }
    }
}