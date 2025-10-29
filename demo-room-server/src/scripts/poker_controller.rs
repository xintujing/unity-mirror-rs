use crate::metadata_settings::metadata_poker_controller::MetadataPokerController;
use crate::poker_helper::{GeneralSuit, Poker, PokerHelper, PokerRank};
use crate::scripts::network_manager_ext::NetworkManagerExt;
use crate::scripts::network_manager_ext_status::StageStatus;
use crate::scripts::player_ext::{PlayerExt, PlayerIdentity};
use crate::scripts::settlement::PlayType;
use unity_mirror_rs::commons::action::SelfMutAction;
use unity_mirror_rs::macro_namespace::*;
use unity_mirror_rs::macro_network_behaviour::*;
use unity_mirror_rs::mirror::{NetworkManager, Operation, SyncList};
use unity_mirror_rs::{data_type_deserialize, data_type_serialize};

#[namespace(prefix = "DaDaA.Scripts")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ShuffleType {
    Simple = 0,
    Symmetry,
}

#[namespace(prefix = "DaDaA.Scripts")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Res {
    Invalid = 0,
    Success,
    NoPermission,
}

data_type_serialize!(
    (
       ShuffleType,Res
    ),
    |value, writer| writer.write_blittable(*value)
);
data_type_deserialize!(
    (
        ShuffleType,Res
    ),
    {|reader| reader.read_blittable()}
);

#[namespace(prefix = "DaDaA.Scripts")]
#[network_behaviour(parent(NetworkBehaviour), metadata(MetadataPokerController))]
pub struct PokerController {
    #[sync_obj]
    pub poker_cards: SyncList<u16>,
    #[sync_obj]
    pub fake_poker_cards: SyncList<u16>,
    pub trump_cards: Vec<u16>,
}

// hook
impl PokerController {
    // 监听扑克变化
    fn on_poker_cards_changed(&mut self, operation: Operation, index: usize, value: u16) {
        match operation {
            Operation::OpAdd => {
                self.fake_poker_cards.add(value);
            }
            Operation::OpSet | Operation::OpInsert => {
                self.fake_poker_cards.insert(index, value);
            }
            Operation::OpRemoveAt => {
                self.fake_poker_cards.remove_at(index);
            }
            Operation::OpClear => {
                self.fake_poker_cards.clear();
            }
        }
    }
}

impl PokerController {
    // 检查是否拥有所有牌
    fn has_all_cards(&self, cards: &[u16]) -> bool {
        for card in cards.iter() {
            if !self.poker_cards.contains(card) {
                return false;
            }
        }
        true
    }

    fn has_permissions(&self, status: &[StageStatus]) -> bool {
        if let Some(obj) = self.game_object.upgrade() {
            if let Some(player_ext) = obj.try_get_component2::<PlayerExt>() {
                for status in status.iter() {
                    if player_ext.get_player_stage_status() == status {
                        return true;
                    }
                }
            } else {
                log::error!("Player({:?}) does not have a PlayerExt", self);
            }
        } else {
            log::error!("PokerController's GameObject has been destroyed");
        }
        false
    }
}

// command/rpc
impl PokerController {
    // 洗牌
    #[command(PokerController)]
    fn cmd_shuffle(&mut self, shuffle_type: ShuffleType) {
        if !self.has_permissions(&vec![StageStatus::Shuffle]) {
            log::warn!("无权限 - 洗牌");
            self.rpc_res(StageStatus::Deal, Res::NoPermission);
            return;
        }

        if let Some(nme) = NetworkManager::singleton_mut::<NetworkManagerExt>() {
            log::debug!("{:?} before {:?}",shuffle_type, nme.collect_pokers);
            match shuffle_type {
                ShuffleType::Simple => {
                    nme.collect_pokers = PokerHelper::shuffle_simple(nme.collect_pokers.as_slice(), 1)
                }
                ShuffleType::Symmetry => {
                    nme.collect_pokers = PokerHelper::shuffle_symmetry(nme.collect_pokers.as_slice(), 1);
                }
            }
            log::debug!("{:?}  after {:?}", shuffle_type, nme.collect_pokers);
        }
    }

    // 结束洗牌
    #[command(PokerController)]
    pub fn cmd_end_shuffle(&mut self) {
        if !self.has_permissions(&vec![StageStatus::Shuffle]) {
            log::warn!("无权限 - 洗牌");
            self.rpc_res(StageStatus::Deal, Res::NoPermission);
            return;
        }

        // 更改所有玩家身份函数
        let set_all_player_ext_identity = |identity: PlayerIdentity| {
            if let Some(nme) = NetworkManager::singleton_mut::<NetworkManagerExt>() {
                for (index, st) in nme.seat_infos.iter().enumerate() {
                    if index == 0 {
                        continue;
                    }
                    NetworkManagerExt::set_player_ext_identity(st.game_object.clone(), identity);
                }
            }
        };

        if let Some(obj) = self.game_object.upgrade() {
            if let Some(mut player_ext) = obj.try_get_component2::<PlayerExt>() {
                if let Some(nme) = NetworkManager::singleton_mut::<NetworkManagerExt>() {
                    // 设置游戏状态为切牌
                    nme.set_game_state(StageStatus::Cut);
                    // 把所有玩家的身份设置为 None
                    set_all_player_ext_identity(PlayerIdentity::Ordinary);
                    // 设置当前玩家状态为 None
                    player_ext.set_player_stage_status(StageStatus::None);
                    // 上家玩家的索引
                    let pre_player_index = ((player_ext.get_nme_index() + 4) % 5) as usize + 1;
                    // 把洗牌玩家的上家状态更改为切牌
                    NetworkManagerExt::set_player_ext_status(nme.seat_infos[pre_player_index].game_object.clone(), StageStatus::Cut);
                }
            }
        }
    }

    // 切牌 0：不切牌 other: 切牌位置
    #[command(PokerController)]
    pub fn cmd_cut(&mut self, index: u16) {
        if !self.has_permissions(&vec![StageStatus::Cut]) {
            log::warn!("无权限 - 切牌");
            self.rpc_res(StageStatus::Cut, Res::NoPermission);
            return;
        }

        if let Some(nme) = NetworkManager::singleton_mut::<NetworkManagerExt>() {
            log::debug!("Cutting cards at index before: {} {:?}", index, nme.collect_pokers);
            // 切牌
            nme.collect_pokers = PokerHelper::cut_cards(nme.collect_pokers.as_slice(), index as usize);
            // 设置玩家状态为无
            NetworkManagerExt::set_player_ext_status(self.game_object.clone(), StageStatus::None);
            log::debug!("Cutting cards at index  after: {} {:?}", index, nme.collect_pokers);
            // 设置游戏状态为发牌
            nme.set_game_state(StageStatus::Deal);
            // 发牌
            nme.deal_cards();
        }
    }

    // 拿起底牌 0: 全部 其它: n 张
    #[command(PokerController)]
    fn cmd_take_trump_cards(&mut self, n: u16) {
        if self.trump_cards.len() == 0 {
            return;
        }

        // 底牌不够
        if n > self.trump_cards.len() as u16 {
            log::warn!("Requested more trump cards than available: {} > {}", n, self.trump_cards.len());
            return;
        }

        // 反转成为从上往下拿
        self.trump_cards.reverse();

        // 全部
        if n == 0 {
            while let Some(poker) = self.trump_cards.pop() {
                self.poker_cards.add(poker);
            }
            return;
        }

        // 拿 n 张
        for _ in 0..n {
            if let Some(poker) = self.trump_cards.pop() {
                self.poker_cards.add(poker);
            } else {
                log::error!("Not enough trump cards to take {} cards", n);
                break;
            }
        }
    }

    // 亮 A
    #[command(PokerController)]
    pub fn cmd_bright_a(&mut self, cards: &[u16]) {
        if !self.has_permissions(&vec![StageStatus::BrightAce, StageStatus::DoubleAceRev]) {
            log::warn!("无权限 - 亮 A");
            self.rpc_res(StageStatus::BrightAce, Res::NoPermission);
            return;
        }

        if cards.len() > 2 {
            log::error!("最多是双 ACE，不能亮超过两张的牌");
            self.rpc_res(StageStatus::BrightAce, Res::Invalid);
            return;
        }

        // 闭包 start

        // 返还所有人底牌
        let return_all_player_trump_cards = |n: u16| {
            if let Some(nme) = NetworkManager::singleton_mut::<NetworkManagerExt>() {
                for sl in nme.seat_infos.iter() {
                    // 如果有玩家持有底牌
                    if let Some(obj) = sl.game_object.upgrade() {
                        if let Some(mut poker_controller) = obj.try_get_component2::<PokerController>() {
                            if !poker_controller.trump_cards.is_empty() {
                                poker_controller.cmd_take_trump_cards(n);
                            }
                        }
                    }
                }
            }
        };

        // 切到 DoubleAceRev 状态
        let to_double_ace_rev = |this: &mut PokerController| -> bool{
            if let Some(nme) = NetworkManager::singleton_mut::<NetworkManagerExt>() {
                // 游戏状态改为双 A 反
                nme.set_game_state(StageStatus::DoubleAceRev);
                for sl in nme.seat_infos.iter() {
                    // 跳过自己
                    if sl.game_object.ptr_eq(&this.game_object) {
                        continue;
                    }
                    // 如果有玩家持有相同两张 ACE
                    if let Some(obj) = sl.game_object.upgrade() {
                        if let Some(poker_controller) = obj.try_get_component2::<PokerController>() {
                            if PokerHelper::has_two_ace(poker_controller.poker_cards.iter().cloned().collect::<Vec<u16>>().as_slice()) {
                                // 持有相同两张牌的玩家状态改为双 A 反
                                NetworkManagerExt::set_player_ext_status(sl.game_object.clone(), StageStatus::DoubleAceRev);
                                return true;
                            }
                        }
                    }
                }
            }
            return false;
        };

        // 切到 ChuAce 状态
        let to_chu_ace = |this: &mut PokerController| -> bool {
            // 转到持有相同一张牌的玩家 状态改为 ChuAce
            if let Some(nme) = NetworkManager::singleton_mut::<NetworkManagerExt>() {
                // 游戏状态改为 ChuAce
                nme.set_game_state(StageStatus::ChuAce);
                for sl in nme.seat_infos.iter() {
                    // 如果有玩家持有和大 A 相同的牌
                    if let Some(obj) = sl.game_object.upgrade() {
                        if let Some(poker_controller) = obj.try_get_component2::<PokerController>() {
                            // 如果是亮 A 的玩家
                            if poker_controller.has_all_cards(nme.big_ace_s().as_slice()) {
                                continue;
                            }
                            if PokerHelper::has_n_poker(poker_controller.poker_cards.iter().cloned().collect::<Vec<u16>>().as_slice(), &Poker::new_with_id(nme.big_ace_s()[0]), 1) {
                                // 持有相同一张牌的玩家状态改为 ChuAce
                                NetworkManagerExt::set_player_ext_status(sl.game_object.clone(), StageStatus::ChuAce);
                                return true;
                            }
                        }
                    }
                }
            }
            return false;
        };

        // 亮 A 的函数
        let f_bright_a = |this: &mut PokerController, cards: &[u16], is_orphan_ace: bool| -> bool {
            // 判断是否是 ACE
            if (cards[0] & PokerHelper::RANK_MASK) as u8 != PokerRank::Ace(GeneralSuit::Heart).to_u8() {
                log::error!("必须是 ACE 才能亮");
                return false;
            }

            // 更改所有人状态
            NetworkManagerExt::set_all_player_ext_status(StageStatus::None);

            // 返还底牌
            return_all_player_trump_cards(0);

            // 更改身份
            NetworkManagerExt::set_player_ext_identity(this.game_object.clone(), PlayerIdentity::SingleBright);

            if let Some(nme) = NetworkManager::singleton_mut::<NetworkManagerExt>() {
                // 同步亮的牌
                nme.big_ace_s_add_range(cards);

                // 修正结算器
                nme.set_play_type(PlayType::T2_3);

                let has_2_big_ace_poker = PokerHelper::has_n_poker(this.poker_cards.iter().cloned().collect::<Vec<u16>>().as_slice(), &Poker::new_with_id(nme.big_ace_s()[0]), 2);

                // 是否是孤 ACE
                match is_orphan_ace {
                    true => {
                        match has_2_big_ace_poker {
                            true => {
                                // 修正自己的身份
                                NetworkManagerExt::set_player_ext_identity(this.game_object.clone(), PlayerIdentity::SingleBrightDark);
                                // 修正结算器
                                nme.set_play_type(PlayType::T1_4);
                            }
                            false => {
                                for sl in nme.seat_infos.iter() {
                                    if sl.game_object.ptr_eq(&this.game_object) {
                                        continue;
                                    }
                                    // 如果有玩家持有和大 A 相同的牌
                                    if let Some(obj) = sl.game_object.upgrade() {
                                        if let Some(mut player_ext) = obj.try_get_component2::<PlayerExt>() {
                                            player_ext.set_player_ext_identity(PlayerIdentity::SingleDark);
                                        }
                                    }
                                }
                            }
                        }
                        NetworkManagerExt::set_player_ext_status(this.game_object.clone(), StageStatus::PlayCards);
                    }
                    false => {
                        // 更改自己状态为 WaitingCountdown
                        match has_2_big_ace_poker {
                            true => {
                                // 修正自己的身份
                                NetworkManagerExt::set_player_ext_identity(this.game_object.clone(), PlayerIdentity::SingleBrightDark);
                                // 设置玩家状态为 WaitPlayCards
                                NetworkManagerExt::set_player_ext_status(this.game_object.clone(), StageStatus::WaitPlayCards);
                                log::debug!("单张 {:?}，等待倒计时",PokerHelper::ids_to_cards(cards));
                            }
                            false => {
                                if to_double_ace_rev(this) {
                                    return true;
                                }
                                if to_chu_ace(this) {
                                    return true;
                                }
                            }
                        };
                    }
                }
            }
            return true;
        };


        // 亮双 ACE 的函数
        let f_bright_double_a = |this: &mut PokerController, cards: &[u16], is_double_inverse: bool| -> bool {
            // 判断是否是双 ACE
            if (cards[0] & PokerHelper::POKER_MASK) != (cards[1] & PokerHelper::POKER_MASK)
                || (cards[0] & PokerHelper::RANK_MASK) != PokerRank::Ace as u16
                || (cards[1] & PokerHelper::RANK_MASK) != PokerRank::Ace as u16 {
                log::error!("双 ACE 必须是两张相同的 ACE");
                return false;
            }

            // 更改所有人状态
            NetworkManagerExt::set_all_player_ext_status(StageStatus::None);

            // 返还底牌
            return_all_player_trump_cards(0);

            // 更改身份
            match is_double_inverse {
                true => {
                    NetworkManagerExt::set_player_ext_identity(this.game_object.clone(), PlayerIdentity::DoubleInverse);
                }
                false => {
                    NetworkManagerExt::set_player_ext_identity(this.game_object.clone(), PlayerIdentity::DoubleBright);
                }
            }

            if let Some(nme) = NetworkManager::singleton_mut::<NetworkManagerExt>() {
                // 同步亮的牌
                nme.big_ace_s_add_range(cards);

                // 修正结算器
                nme.set_play_type(PlayType::T1_4);
            }
            // 更改自己状态为 PlayCards
            NetworkManagerExt::set_player_ext_status(this.game_object.clone(), StageStatus::PlayCards);
            true
        };

        // 孤 ACE 亮牌
        let f_orphan_ace = || {
            if let Some(nme) = NetworkManager::singleton_mut::<NetworkManagerExt>() {
                let mut dont_bright_a_count = 0;
                for sl in nme.seat_infos.iter() {
                    if let Some(obj) = sl.game_object.get() {
                        if let Some(mut player_ext) = obj.try_get_component2::<PlayerExt>() {
                            if player_ext.get_player_stage_status() == &StageStatus::None {
                                dont_bright_a_count += 1;
                            }
                        }
                    }
                }
                log::debug!("孤 ACE 亮牌, 不亮 A 的玩家数量: {} {}", dont_bright_a_count, nme.seat_infos.len() -1);
                if nme.game_state() == StageStatus::BrightAce && dont_bright_a_count == nme.seat_infos.len() - 1 {
                    for sl in nme.seat_infos.iter() {
                        if let Some(obj) = sl.game_object.upgrade() {
                            if let Some(mut poker_controller) = obj.try_get_component2::<PokerController>() {
                                let (mut r#do, mut id) = (false, 0);
                                // 找出 ACE
                                for p_id in poker_controller.poker_cards.iter() {
                                    if (*p_id & PokerHelper::RANK_MASK) as u8 == PokerRank::Ace(GeneralSuit::Heart).to_u8() {
                                        r#do = true;
                                        id = *p_id;
                                        break;
                                    }
                                }
                                if r#do {
                                    // 亮 A
                                    f_bright_a(&mut poker_controller, &[id], true);

                                    // 同步亮的牌
                                    nme.big_ace_s_add_range(&[id]);

                                    log::debug!("所有玩家都不亮, 亮孤 ACE: {:?}", PokerHelper::ids_to_cards(&[id]));
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        };

        // 闭包 end

        if let Some(nme) = NetworkManager::singleton_mut::<NetworkManagerExt>() {
            // 如果游戏状态不是亮 A
            if nme.game_state() != StageStatus::BrightAce && nme.game_state() != StageStatus::DoubleAceRev {
                log::error!("当前游戏状态不是亮 A 或双 A 反，无法执行亮 A 操作");
                self.rpc_res(StageStatus::None, Res::Invalid);
                return;
            }

            // 有人亮过双 ACE
            if nme.big_ace_s().len() == 2 {
                log::error!("已经有玩家亮过双 ACE，不能再亮双 ACE");
                self.rpc_res(StageStatus::BrightAce, Res::Invalid);
                return;
            }

            // 有人亮过单 ACE 且要双 A 反
            if cards.len() == 2 && nme.big_ace_s().len() == 1 {
                log::error!("有人亮过单 ACE 且要双 A 反");
                f_bright_double_a(self, cards, true);
                return;
            }


            // 不亮 ACE
            if cards.is_empty() && nme.game_state() == StageStatus::BrightAce {
                if let Some(obj) = self.game_object.get() {
                    if let Some(mut player_ext) = obj.try_get_component2::<PlayerExt>() {
                        // 当前玩家
                        {
                            log::debug!("不亮 A");
                            // 返还底牌 0 全部
                            self.cmd_take_trump_cards(0);
                            // 设置玩家状态为无
                            player_ext.set_player_stage_status(StageStatus::None);
                        }
                    }
                }
                // TODO 都不亮
                f_orphan_ace();
                return;
            }

            // 不双 ACE 反
            if cards.is_empty() && nme.game_state() == StageStatus::DoubleAceRev {
                log::debug!("不亮双 ACE 反，游戏状态改为杵 A");
                // 更改自己状态为 None
                NetworkManagerExt::set_player_ext_status(self.game_object.clone(), StageStatus::None);
                // 处理是否有人可以杵 A
                to_chu_ace(self);
                return;
            }
        }

        log::debug!("亮: {:?} {:?} {} {}", cards,PokerHelper::ids_to_cards(cards),cards[0] & PokerHelper::RANK_MASK,PokerRank::Ace(GeneralSuit::Heart).to_u8());

        // 单张 ACE
        if cards.len() == 1 && !f_bright_a(self, cards, false) {
            return;
        }

        // 双张 ACE
        if cards.len() == 2 && !f_bright_double_a(self, cards, false) {
            return;
        }
    }

    // 结束 WaitPlayCards 状态
    #[command(PokerController)]
    pub fn cmd_end_wait_play_cards(&mut self) {
        if !self.has_permissions(&vec![StageStatus::WaitPlayCards]) {
            log::warn!("无权限 - 结束等待出牌");
            self.rpc_res(StageStatus::WaitPlayCards, Res::NoPermission);
            return;
        }

        // 设置玩家状态为 PlayCards
        NetworkManagerExt::set_player_ext_status(self.game_object.clone(), StageStatus::PlayCards);
    }

    // chu A
    #[command(PokerController)]
    pub fn cmd_pestle_a(&mut self, sure: bool) {
        if !self.has_permissions(&vec![StageStatus::ChuAce]) {
            log::warn!("无权限 - Chu A");
            self.rpc_res(StageStatus::ChuAce, Res::NoPermission);
            return;
        }

        if let Some(nme) = NetworkManager::singleton_mut::<NetworkManagerExt>() {
            if nme.game_state() != StageStatus::ChuAce {
                log::error!("当前游戏状态不是出 A，无法执行杵 A 操作");
                return;
            }

            match sure {
                true => { // 杵 A
                    // 游戏状态改为出牌
                    nme.set_game_state(StageStatus::PlayCards);
                    // 修正自己的身份
                    NetworkManagerExt::set_player_ext_identity(self.game_object.clone(), PlayerIdentity::SingleDarkPestle);
                    // 设置玩家状态为出牌
                    NetworkManagerExt::set_player_ext_status(self.game_object.clone(), StageStatus::PlayCards);
                    log::debug!("杵 A，游戏状态改为出牌");
                    // 修正亮 A 的玩家身份
                    for sl in nme.seat_infos.iter() {
                        // 设置当前操作玩家
                        if sl.game_object.ptr_eq(&self.game_object) {
                            continue;
                        }
                        // 如果有玩家持有相同的牌
                        if let Some(obj) = sl.game_object.upgrade() {
                            if let Some(mut player_ext) = obj.try_get_component2::<PlayerExt>() {
                                if *player_ext.get_player_ext_identity() == PlayerIdentity::SingleBright {
                                    player_ext.set_player_ext_identity(PlayerIdentity::SingleBrightQuilt);
                                    log::debug!("修正玩家身份为 SingleBrightQuilt");
                                }
                            }
                        }
                    }
                }
                false => { // 不杵 A
                    // 游戏状态改为出牌
                    nme.set_game_state(StageStatus::PlayCards);
                    // 修正自己的身份
                    NetworkManagerExt::set_player_ext_identity(self.game_object.clone(), PlayerIdentity::SingleDark);
                    // 当前操作玩家状态改为 None
                    NetworkManagerExt::set_player_ext_status(self.game_object.clone(), StageStatus::None);
                    for sl in nme.seat_infos.iter() {
                        // 设置当前操作玩家状态为 None
                        if sl.game_object.ptr_eq(&self.game_object) {
                            continue;
                        }
                        if let Some(obj) = sl.game_object.upgrade() {
                            if let Some(mut player_ext) = obj.try_get_component2::<PlayerExt>() {
                                if *player_ext.get_player_ext_identity() == PlayerIdentity::SingleBright {
                                    // 设置玩家状态为出牌
                                    player_ext.set_player_stage_status(StageStatus::PlayCards);
                                    log::debug!("不杵 A，游戏状态改为出牌");
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // 出牌
    #[command(PokerController)]
    pub fn cmd_play_cards(&mut self, cards: &[u16]) {
        if !self.has_permissions(&vec![StageStatus::PlayCards]) {
            log::warn!("无权限 - 出牌");
            self.rpc_res(StageStatus::PlayCards, Res::NoPermission);
            return;
        }

        // 判断是否有牌
        if !self.has_all_cards(cards) {
            log::warn!("没有这些牌，无法出牌: {:?}", PokerHelper::ids_to_cards(cards));
            self.rpc_res(StageStatus::PlayCards, Res::Invalid);
            return;
        }

        if let Some(nme) = NetworkManager::singleton_mut::<NetworkManagerExt>() {
            if cards.is_empty() && nme.last_pokers().unwrap().count() == 0 {
                log::debug!("没有上家出牌或没有人管自己出的牌，必须出牌");
                self.rpc_res(StageStatus::PlayCards, Res::Invalid);
                return;
            }

            // 不出
            if cards.is_empty() && nme.last_pokers().unwrap().count() > 0 {
                log::debug!("不出牌");
                nme.next_player(self.game_object.clone(), StageStatus::None);
                return;
            }

            // 转化为 Poker
            let pokers = PokerHelper::ids_to_cards(&cards);

            let mut last_pokers = Vec::new();
            for id in nme.last_pokers().unwrap().iter() {
                last_pokers.push(Poker::new_with_id(*id));
            }

            // 比较双方牌
            let (can, get_cards) = PokerHelper::calculate(&last_pokers, &pokers, nme.big_ace_suit());

            log::debug!("上家: {:?} 当前: {:?} 是否能管上: {:?}  取到的牌: {:?}", last_pokers,PokerHelper::ids_to_cards(cards), can, get_cards);

            // 取走的牌的 ids
            let get_cards = PokerHelper::cards_to_ids(&get_cards);

            // 如果不能出牌
            if !can {
                self.rpc_res(StageStatus::PlayCards, Res::Invalid);
                return;
            }

            // 收集出的牌
            nme.collect_pokers.extend(pokers);

            // 移除掉已经出的牌
            self.poker_cards.remove_range(cards);

            // 如果玩家手牌出完了
            if self.poker_cards.count() == 0 {
                if let Some(obj) = self.game_object.upgrade() {
                    if let Some(player_ext) = obj.try_get_component2::<PlayerExt>() {
                        match player_ext.get_player_ext_identity() {
                            PlayerIdentity::Ordinary => {
                                nme.settlement.ordinary_out_count += 1;
                                log::debug!("普通玩家出完牌，增加数量 {}", nme.settlement.ordinary_out_count);
                            }
                            _ => {
                                nme.settlement.ace_out_count += 1;
                                log::debug!("大 A 玩家出完牌，增加数量 {}", nme.settlement.ace_out_count);
                            }
                        }
                    }
                }
            }

            // 修正玩家身份
            if let Some(game_object) = self.game_object.get() {
                if let Some(mut player_ext) = game_object.try_get_component2::<PlayerExt>() {
                    if *player_ext.get_player_ext_identity() == PlayerIdentity::SingleDark && PokerHelper::has_n_poker(cards, &Poker::new_with_id(nme.big_ace_s()[0]), 1) {
                        // 修正自己的身份
                        NetworkManagerExt::set_player_ext_identity(self.game_object.clone(), PlayerIdentity::SingleDark2Bright);
                        log::debug!("修正玩家身份为 SingleDark2Bright");
                    }
                }
            }

            if !get_cards.is_empty() {
                // 添加取回的牌
                self.poker_cards.add_range(get_cards.to_vec());

                // 移除掉被取走的牌
                nme.collect_pokers.retain(|&item| !get_cards.contains(&item.id()));
            }

            // 记录出的牌
            nme.last_pokers().unwrap().clear();
            nme.last_pokers().unwrap().add_range(cards.to_vec());

            // 记录出牌玩家
            nme.last_player = self.game_object.clone();

            {
                match self.poker_cards.count() > 0 {
                    true => {
                        // 之后该是谁出牌
                        nme.next_player(self.game_object.clone(), StageStatus::None);
                    }
                    false => {
                        // TODO 结算
                        if nme.settlement.can_settlement() {
                            log::debug!("结算");
                            nme.set_game_state(StageStatus::Settlement);
                            NetworkManagerExt::set_all_player_ext_status(StageStatus::Settlement);
                            return;
                        }
                        // 之后该是谁出牌
                        nme.next_player(self.game_object.clone(), StageStatus::Waiting);
                    }
                }
            }

            // 通知所有人出牌结果
            self.rpc_res(StageStatus::PlayCards, Res::Success);
        }
    }

    // 出牌结果
    #[client_rpc]
    fn rpc_res(&mut self, player_ext_status: StageStatus, res: Res) {}
}

impl MonoBehaviour for PokerController {
    fn awake(&mut self) {
        self.poker_cards.on_change = SelfMutAction::new(self.weak.clone(), Self::on_poker_cards_changed);
    }

    fn on_destroy(&mut self) {
        self.poker_cards.on_change = SelfMutAction::default();
    }
}

impl TNetworkBehaviour for PokerController {
    fn new(_: RevelWeak<GameObject>, _: &MetadataNetworkBehaviourWrapper) -> Self
    where
        Self: Sized,
    {
        let poker_controller = Self::default();

        poker_controller
    }
}

impl PokerControllerOnChangeCallback for PokerController {}