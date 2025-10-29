use crate::metadata_settings::metadata_network_manager_ext::MetadataNetworkManagerExt;
use crate::poker_helper::{GeneralSuit, Poker, PokerHelper};
use crate::scripts::network_manager_ext_status::NetworkManagerExtStatus;
use crate::scripts::network_manager_ext_status::StageStatus;
use crate::scripts::player_ext::{PlayerExt, PlayerIdentity};
use crate::scripts::poker_controller::PokerController;
use crate::scripts::settlement::{PlayType, Settlement};
use std::error::Error;
use unity_mirror_rs::commons::action::SelfMutAction;
use unity_mirror_rs::macro_namespace::*;
use unity_mirror_rs::macro_network_manager::*;
use unity_mirror_rs::metadata_settings::Metadata;
use unity_mirror_rs::mirror::{NetworkConnectionToClient, NetworkManager, NetworkServer, SyncList, TransportError};
use unity_mirror_rs::unity_engine::{Transform, WorldManager};

#[derive(Default, Clone)]
pub struct SeatLocation {
    pub name: String,
    pub game_object: RevelWeak<GameObject>,
    pub transform: Transform,
}

#[namespace(prefix = "DaDaA.Scripts")]
#[network_manager(parent(NetworkManager))]
pub struct NetworkManagerExt {
    pub min_players: u32,
    pub player_ext_s: Vec<RevelWeak<Box<PlayerExt>>>,
    pub seat_infos: [SeatLocation; 6],
    network_manager_ext_status_prefab: String,
    pub network_manager_ext_status: RevelWeak<Box<NetworkManagerExtStatus>>,
    // 0: 桌面侧
    pub collect_pokers: Vec<Poker>,
    // 最后打牌的玩家
    pub last_player: RevelWeak<GameObject>,
    // 结算器
    pub settlement: Settlement,
}

impl MonoBehaviour for NetworkManagerExt {
    fn awake(&mut self) {
        self.parent.awake();
        (self.collect_pokers, _) = PokerHelper::generate_cards(2);
    }

    fn on_validate(&mut self) {
        self.parent.on_validate();

        // always <= maxConnections
        self.min_players = self.min_players.min(self.max_connections as u32);

        // always >= 0
        self.min_players = self.min_players.max(0);

        if self.player_prefab.is_empty() || self.player_prefab == "" {
            log::error!("Player prefab is not set. Please set it in the inspector.");
        }
    }

    fn start(&mut self) {
        self.parent.start();
    }

    fn update(&mut self) {
        self.parent.update();
    }

    fn late_update(&mut self) {
        self.parent.late_update();
    }

    fn on_destroy(&mut self) {
        self.parent.on_destroy();
    }
}

// Actions
impl NetworkManagerExt {
    /// 当服务器启动时调用
    fn on_start_server(&mut self) {
        // 生成 NetworkManagerExtStatus
        if let Some(prefab) = Metadata::get_prefab(&self.network_manager_ext_status_prefab) {
            let game_object = GameObject::instantiate(&prefab);
            match game_object.try_get_component2::<NetworkManagerExtStatus>() {
                None => {
                    log::error!("NetworkManagerExtStatus prefab does not have NetworkManagerExtStatus component: {}",game_object.name);
                }
                Some(rev_box) => {
                    self.network_manager_ext_status = rev_box.downgrade();
                    // 生成
                    NetworkServer::spawn(rev_box.game_object.clone());
                    // 不要在场景切换时销毁
                    WorldManager::dont_destroy_object(game_object);
                }
            }
        } else {
            log::error!("NetworkManagerExtStatus prefab not found: {}", self.network_manager_ext_status_prefab);
        }
    }

    /// 当服务器停止时调用
    fn on_stop_server(&mut self) {
        // 销毁 NetworkManagerExtStatus
        if let Some(network_manager_ext_status) = self.network_manager_ext_status.upgrade() {
            NetworkServer::un_spawn(network_manager_ext_status.game_object.clone())
        }
    }

    /// 新客户连接时在服务器上调用
    fn on_server_connect(&mut self, mut connection: RevelArc<Box<NetworkConnectionToClient>>) {}

    /// 客户准备就绪时在服务器上打电话
    pub fn on_server_ready(&mut self, connection: RevelArc<Box<NetworkConnectionToClient>>) {}

    /// 当服务器更改场景时调用
    fn server_change_scene(&mut self, scene_name: String) {}

    /// 当服务器场景更改完成时调用
    fn on_server_scene_changed(&mut self, scene_name: String) {
        self.parent.on_server_scene_changed_default(scene_name.clone());
    }

    /// 当客户端添加使用 NetworkClient.AddPlayer 的新播放器时，请在服务器上调用
    fn on_server_add_player(&mut self, connection: RevelArc<Box<NetworkConnectionToClient>>) {
        self.parent.on_server_add_player_default(connection);
    }

    /// 当服务器发生错误时调用
    fn on_server_error(&mut self, _connection: RevelArc<Box<NetworkConnectionToClient>>, _error: TransportError, _reason: String) {}

    /// 当服务器传输异常时调用
    fn on_server_transport_exception(&mut self, _connection: RevelArc<Box<NetworkConnectionToClient>>, _error: Box<dyn Error>) {}

    /// 当客户端断开连接时在服务器上调用
    fn on_server_disconnect(&mut self, connection: RevelArc<Box<NetworkConnectionToClient>>) {
        if let Some(network_manager_ext_status) = self.network_manager_ext_status.get() {
            network_manager_ext_status.set_all_players_ready(false);
        }

        self.parent.on_server_disconnect_default(connection);
    }
}

impl NetworkManagerExtInitialize for NetworkManagerExt {
    fn initialize(&mut self, metadata: &MetadataNetworkManagerWrapper) {
        let config = metadata.get::<MetadataNetworkManagerExt>();
        self.min_players = config.min_players;

        // network_manager_ext_status_prefab
        self.network_manager_ext_status_prefab = config.network_manager_ext_status.game_object.asset_path.clone();

        // Actions
        self.on_start_server = SelfMutAction::new(self.weak.clone(), Self::on_start_server);
        self.on_stop_server = SelfMutAction::new(self.weak.clone(), Self::on_stop_server);
        // self.on_server_connect = SelfMutAction::new(self.weak.clone(), Self::on_server_connect);
        // self.on_server_ready = SelfMutAction::new(self.weak.clone(), Self::on_server_ready);
        // self.server_change_scene = SelfMutAction::new(self.weak.clone(), Self::server_change_scene);
        self.on_server_scene_changed = SelfMutAction::new(self.weak.clone(), Self::on_server_scene_changed);
        self.on_server_add_player = SelfMutAction::new(self.weak.clone(), Self::on_server_add_player);
        // self.on_server_error = SelfMutAction::new(self.weak.clone(), Self::on_server_error);
        // self.on_server_transport_exception = SelfMutAction::new(self.weak.clone(), Self::on_server_transport_exception);
        self.on_server_disconnect = SelfMutAction::new(self.weak.clone(), Self::on_server_disconnect);
    }
}

impl NetworkManagerExt {
    pub fn all_players_ready(&self) -> bool {
        if let Some(network_manager_ext_status) = self.network_manager_ext_status.get() {
            return *network_manager_ext_status.get_all_players_ready();
        }
        log::error!("NetworkManagerExtStatus is not initialized.");
        false
    }

    pub fn set_all_players_ready(&mut self, now_ready: bool) {
        if let Some(network_manager_ext_status) = self.network_manager_ext_status.get() {
            let was_ready = *network_manager_ext_status.get_all_players_ready();
            if was_ready != now_ready {
                network_manager_ext_status.set_all_players_ready(now_ready);
                if now_ready {
                    self.on_room_server_players_ready();
                } else {
                    self.on_room_server_players_not_ready();
                }
            }
        } else {
            log::error!("NetworkManagerExtStatus is not initialized.");
        }
    }

    pub fn set_game_state(&mut self, state: StageStatus) {
        if let Some(network_manager_ext_status) = self.network_manager_ext_status.get() {
            network_manager_ext_status.set_game_stage_state(state);
        } else {
            log::error!("NetworkManagerExtStatus is not initialized.");
        }
    }

    pub fn game_state(&self) -> StageStatus {
        if let Some(network_manager_ext_status) = self.network_manager_ext_status.get() {
            return *network_manager_ext_status.get_game_stage_state();
        }
        log::error!("NetworkManagerExtStatus is not initialized.");
        StageStatus::None
    }

    pub fn big_ace_s_add_range(&mut self, cards: &[u16]) {
        if let Some(network_manager_ext_status) = self.network_manager_ext_status.get() {
            network_manager_ext_status.big_ace_s.clear();
            network_manager_ext_status.big_ace_s.add_range(cards.to_vec());
        } else {
            log::error!("NetworkManagerExtStatus is not initialized.");
        }
    }

    pub fn big_ace_s(&self) -> Vec<u16> {
        let mut cards = Vec::new();
        if let Some(network_manager_ext_status) = self.network_manager_ext_status.get() {
            for id in network_manager_ext_status.big_ace_s.iter() {
                cards.push(*id);
            }
        } else {
            log::error!("NetworkManagerExtStatus is not initialized.");
        }
        cards
    }

    // big_ace_suit
    pub fn big_ace_suit(&self) -> GeneralSuit {
        let big_ace_s = self.big_ace_s();
        for id in big_ace_s.iter() {
            if let Ok(suit) = GeneralSuit::try_from((id >> 5 & PokerHelper::SUIT_MASK) as u8) {
                return suit;
            }
        }
        log::error!("Big ace suit not found.");
        GeneralSuit::Heart
    }

    pub fn last_pokers(&self) -> Option<&mut SyncList<u16>> {
        if let Some(network_manager_ext_status) = self.network_manager_ext_status.get() {
            return Some(&mut network_manager_ext_status.last_pokers);
        }
        log::error!("NetworkManagerExtStatus is not initialized.");
        None
    }

    pub fn set_last_pokers(&mut self, ids: &[u16]) {
        match self.last_pokers() {
            None => {
                log::error!("last_pokers not found.");
            }
            Some(last_pokers) => {
                last_pokers.clear();
                last_pokers.add_range(ids.to_vec());
            }
        }
    }

    pub fn ready_status_changed(&mut self) {
        let mut current_players = 0;
        let mut ready_players = 0;

        for weak_player_ext in self.player_ext_s.iter() {
            if let Some(player_ext) = weak_player_ext.get() {
                current_players += 1;
                if *player_ext.get_ready_to_begin() {
                    ready_players += 1;
                }
            }
        }

        match current_players == ready_players {
            true => {
                self.check_ready_to_begin();
            }
            false => {
                self.set_all_players_ready(false);
            }
        }
    }

    fn check_ready_to_begin(&mut self) {
        if self.network_scene_name() != self.online_scene {
            return;
        }

        let mut number_of_players = 0;


        for conn in NetworkServer.connections.values_mut() {
            if let Some(identity) = conn.identity.get() {
                if let Some(game_object) = identity.game_object.get() {
                    if let Some(player_ext) = game_object.try_get_component2::<PlayerExt>() {
                        if *player_ext.get_ready_to_begin() {
                            number_of_players += 1;
                        }
                    }
                }
            }
        }

        let enough_ready_players = self.min_players <= 0 || number_of_players >= self.min_players as usize;

        match enough_ready_players {
            true => {
                self.set_all_players_ready(true);
            }
            false => {
                self.set_all_players_ready(false);
            }
        }
    }

    pub fn recalculate_room_player_indices(&mut self) {
        let mut index = 0;
        for weak_player_ext in self.player_ext_s.iter() {
            if let Some(player_ext) = weak_player_ext.get() {
                player_ext.set_nme_index(index);
                index += 1;
            }
        }
    }

    fn on_room_server_players_ready(&mut self) {
        log::info!("All players are ready.");
    }

    fn on_room_server_players_not_ready(&mut self) {
        log::info!("Not all players are ready.");
    }

    pub fn start_game(&mut self) {
        // 检查是否都准备好
        if !self.all_players_ready() {
            log::error!("not all players are ready, cannot start the game.");
            return;
        }

        Self::set_all_player_ext_status(StageStatus::None);

        // 更改游戏状态
        self.set_game_state(StageStatus::Shuffle);

        // 更改大供状态为洗牌
        {
            for sl in self.seat_infos.iter() {
                if let Some(go) = sl.game_object.get() {
                    if let Some(mut player_ext) = go.try_get_component2::<PlayerExt>() {
                        if *player_ext.get_player_ext_identity() == PlayerIdentity::SingleBright {
                            player_ext.set_player_stage_status(StageStatus::Shuffle);
                            break; // 找到庄家
                        }
                    }
                }
            }
        }
    }

    // 更改所有玩家状态
    pub fn set_all_player_ext_status(status: StageStatus) {
        if let Some(nme) = NetworkManager::singleton_mut::<NetworkManagerExt>() {
            for (index, _) in nme.seat_infos.iter().enumerate() {
                if index == 0 {
                    continue;
                }
                NetworkManagerExt::set_player_ext_status(nme.seat_infos[index].game_object.clone(), status);
            }
        }
    }


    // 发牌
    pub fn deal_cards(&mut self) {
        let mut current_seat_index = 1;

        // 重置当前索引到大供位置
        for (index, sl) in self.seat_infos.iter().enumerate() {
            if let Some(go) = sl.game_object.get() {
                if let Some(player_ext) = go.try_get_component2::<PlayerExt>() {
                    if *player_ext.get_player_ext_identity() == PlayerIdentity::SingleBright {
                        current_seat_index = index;
                        break; // 找到大供
                    }
                }
            }
        }

        // 设置房间状态为 BrightAce
        self.set_game_state(StageStatus::BrightAce);

        // 弹出一张
        while let Some(mut poker) = self.collect_pokers.pop() {
            // 重置 used_big_ace 为 false
            poker.used_big_ace = false;
            if let Some(game_obj) = self.seat_infos[current_seat_index].game_object.get() {
                if let Some(mut poker_controller) = game_obj.try_get_component2::<PokerController>() {
                    // 发牌
                    if self.collect_pokers.len() > 5 { // 如果剩余牌大于5张，则发牌
                        // 发牌
                        poker_controller.poker_cards.add(poker.id());
                        // 当发到的牌有 A 时，设置玩家就可以亮 A
                        if PokerHelper::has_any_ace(poker_controller.poker_cards.iter().collect::<Vec<_>>().as_slice())
                        {
                            if let Some(mut player_ext) = game_obj.try_get_component2::<PlayerExt>() {
                                player_ext.set_player_stage_status(StageStatus::BrightAce);
                            }
                        }
                    } else { // 否则将牌作为底牌
                        poker_controller.trump_cards.push(poker.id());
                    }
                }
            }
            // 如果当前索引是最后一个，则回到第一个
            current_seat_index = (current_seat_index) % (self.seat_infos.len() - 1) + 1;
        }

        // 自动把底牌发给没有 A 的玩家
        for sl in self.seat_infos.iter() {
            if let Some(game_obj) = sl.game_object.get() {
                if let Some(mut poker_controller) = game_obj.try_get_component2::<PokerController>() {
                    // 有 A 的玩家不发底牌
                    if PokerHelper::has_any_ace(poker_controller.poker_cards.iter().collect::<Vec<_>>().as_slice())
                    {
                        log::info!("Player at index {} has an Ace, not giving them trump cards.", sl.name);
                        continue;
                    }
                    // 如果没有 A，则发底牌
                    while let Some(poker_id) = poker_controller.trump_cards.pop() {
                        log::info!("Giving trump card {} to player at index {}", poker_id, sl.name);
                        poker_controller.poker_cards.add(poker_id);
                    }
                }
            }
        }
    }

    //
    pub fn set_play_type(&mut self, play_type: PlayType) {
        let old_play_type = self.settlement.play_type;
        // TODO 底分处理
        match old_play_type {
            PlayType::None => {}
            PlayType::T1_4 => {}
            PlayType::T2_3 => {}
        }
        self.settlement.play_type = play_type;
        log::debug!("Play type changed from {:?} to {:?}", old_play_type, play_type);
    }

    // 确定下一个出牌的玩家
    pub fn next_player(&mut self, curr_game_object: RevelWeak<GameObject>, player_ext_status: StageStatus) -> bool {
        let mut curr_index = 0;
        if let Some(obj) = curr_game_object.upgrade() {
            if let Some(mut player_ext) = obj.try_get_component2::<PlayerExt>() {
                player_ext.set_player_stage_status(player_ext_status);
                curr_index = *player_ext.get_nme_index() + 1;
            }
        }
        log::info!("Current player index: {}", curr_index);
        let next_index = (curr_index as usize) % (self.seat_infos.len() - 1) + 1;
        log::info!("Next player index: {}", next_index);
        let next_weak_game_object = self.seat_infos[next_index].game_object.clone();
        if let Some(next_game_object) = next_weak_game_object.get() {
            if let Some(mut next_player_ext) = next_game_object.try_get_component2::<PlayerExt>() {
                // 如果回到了最后一个出牌的玩家，清空最后打出的牌
                let is_same = self.last_player.ptr_eq(&next_weak_game_object);
                if is_same {
                    self.last_pokers().unwrap().clear();
                }
                // 如果玩家有牌
                if *next_player_ext.get_player_stage_status() != StageStatus::Waiting && *next_player_ext.get_player_stage_status() == StageStatus::None {
                    // 设置下一个玩家的状态为出牌
                    Self::set_player_ext_status(next_weak_game_object.clone(), StageStatus::PlayCards);
                    return true;
                }
                // 玩家没牌
                if is_same { // 推风
                    return self.push_wind(curr_game_object);
                }
                return self.next_player(next_weak_game_object.clone(), *next_player_ext.get_player_stage_status());
            }
        }
        false
    }

    // 推风
    pub fn push_wind(&mut self, curr_game_object: RevelWeak<GameObject>) -> bool {
        let mut curr_index: usize = 0;
        let mut curr_identity = PlayerIdentity::Ordinary;
        if let Some(obj) = curr_game_object.upgrade() {
            if let Some(mut player_ext) = obj.try_get_component2::<PlayerExt>() {
                curr_index = *player_ext.get_nme_index() as usize;
                curr_identity = *player_ext.get_player_ext_identity();
            }
        }

        let mut players = Vec::new();

        loop {
            log::info!("Current player index: {}", curr_index);
            curr_index = (curr_index) % (self.seat_infos.len() - 1) + 1;
            log::info!("Next player index: {}", curr_index);
            let next_weak_game_object = self.seat_infos[curr_index].game_object.clone();
            if curr_game_object.ptr_eq(&next_weak_game_object) {
                break;
            }
            if let Some(next_game_object) = next_weak_game_object.get() {
                if let Some(mut player_ext) = next_game_object.try_get_component2::<PlayerExt>() {
                    if *player_ext.get_player_stage_status() == StageStatus::Waiting {
                        continue;
                    }
                    players.push((next_weak_game_object.clone(), *player_ext.get_player_ext_identity()));
                }
            }
        }


        for (next_game_object, next_identity) in players.iter() {
            match curr_identity {
                PlayerIdentity::Ordinary => {
                    // vec![PlayerIdentity::None, PlayerIdentity::SingleDark]

                    if *next_identity == PlayerIdentity::Ordinary || *next_identity == PlayerIdentity::SingleDark {
                        Self::set_player_ext_status(next_game_object.clone(), StageStatus::PlayCards);
                        return true;
                    }
                }
                PlayerIdentity::SingleBrightQuilt => {
                    // vec![PlayerIdentity::SingleDarkPestle]

                    if *next_identity == PlayerIdentity::SingleDarkPestle {
                        Self::set_player_ext_status(next_game_object.clone(), StageStatus::PlayCards);
                        return true;
                    }
                }
                PlayerIdentity::SingleBright => {
                    // 优先选择 SingleDark2Bright
                    'SingleDark2Bright: for (next_game_object, next_identity) in players.iter() {
                        if *next_identity == PlayerIdentity::SingleDark2Bright {
                            Self::set_player_ext_status(next_game_object.clone(), StageStatus::PlayCards);
                            return true;
                        }
                    }

                    // 其次选择 Ordinary
                    'Ordinary: for (next_game_object, next_identity) in players.iter() {
                        if *next_identity == PlayerIdentity::Ordinary {
                            Self::set_player_ext_status(next_game_object.clone(), StageStatus::PlayCards);
                            return true;
                        }
                    }
                }
                PlayerIdentity::SingleDarkPestle => {
                    // vec![PlayerIdentity::SingleBrightQuilt]

                    if *next_identity == PlayerIdentity::SingleBrightQuilt {
                        Self::set_player_ext_status(next_game_object.clone(), StageStatus::PlayCards);
                        return true;
                    }
                }
                PlayerIdentity::SingleDark2Bright => {
                    // vec![PlayerIdentity::SingleBright]

                    if *next_identity == PlayerIdentity::SingleBright {
                        Self::set_player_ext_status(next_game_object.clone(), StageStatus::PlayCards);
                        return true;
                    }
                }
                _ => {
                    log::error!("Shouldn't have been here {:?}", curr_identity);
                    return false;
                }
            };
        }

        false
    }

    pub fn set_player_ext_status(game_object: RevelWeak<GameObject>, player_ext_status: StageStatus) {
        if let Some(obj) = game_object.upgrade() {
            if let Some(mut player_ext) = obj.try_get_component2::<PlayerExt>() {
                player_ext.set_player_stage_status(player_ext_status);
            } else {
                log::error!("GameObject does not have PlayerExt component: {}", obj.name);
            }
        } else {
            log::error!("GameObject is not valid, cannot set PlayerExt status.");
        }
    }

    pub fn set_player_ext_identity(game_object: RevelWeak<GameObject>, player_ext_status: PlayerIdentity) {
        if let Some(obj) = game_object.upgrade() {
            if let Some(mut player_ext) = obj.try_get_component2::<PlayerExt>() {
                // 设置玩家身份
                player_ext.set_player_ext_identity(player_ext_status);
            } else {
                log::error!("GameObject does not have PlayerExt component: {}", obj.name);
            }
        } else {
            log::error!("GameObject is not valid, cannot set PlayerExt identity.");
        }
    }
}