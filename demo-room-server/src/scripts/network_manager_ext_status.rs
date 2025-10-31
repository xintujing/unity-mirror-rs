use crate::metadata_settings::network_manager_ext_status::MetadataNetworkManagerExtStatus;
use unity_mirror_rs::macro_namespace::*;
use unity_mirror_rs::macro_network_behaviour::*;
use unity_mirror_rs::mirror::SyncList;
use unity_mirror_rs::{data_type_deserialize, data_type_serialize};


#[namespace(prefix = "DaDaA.Scripts")]
#[derive(Default, Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
pub enum StageStatus {
    #[default]
    None = 0,
    Shuffle,        // 洗牌
    Cut,            // 切牌
    Deal,           // 发牌
    BrightAce,      // 亮 A
    DoubleAceRev,   // 双 A 反
    ChuAce,         // 杵 A
    WaitPlayCards,  // 等待出牌
    PlayCards,      // 出牌
    Waiting,        // 等待
    Settlement,     // 结算
    // End,         // 结束
}

data_type_serialize!(
    (
       StageStatus
    ),
    |value, writer| writer.write_blittable(*value)
);
data_type_deserialize!(
    (
        StageStatus
    ),
    {|reader| reader.read_blittable()}
);

#[namespace(prefix = "DaDaA.Scripts")]
#[network_behaviour(parent(NetworkBehaviour), metadata(MetadataNetworkManagerExtStatus))]
pub struct NetworkManagerExtStatus {
    #[sync_var]
    pub game_stage_state: StageStatus,
    #[sync_var]
    pub all_players_ready: bool,
    #[sync_obj]
    pub big_ace_s: SyncList<u16>,
    #[sync_obj]
    pub last_pokers: SyncList<u16>,
}

impl NetworkManagerExtStatus {
    pub fn reset(&mut self) {
        self.set_game_stage_state(StageStatus::None);
        self.set_all_players_ready(false);
        self.big_ace_s.reset();
    }
}

impl MonoBehaviour for NetworkManagerExtStatus {}

impl TNetworkBehaviour for NetworkManagerExtStatus {
    fn new(_weak_game_object: RevelWeak<GameObject>, _metadata: &MetadataNetworkBehaviourWrapper) -> Self
    where
        Self: Sized,
    {
        let network_manager_ext_status = Self::default();


        network_manager_ext_status
    }
}

impl NetworkManagerExtStatusOnChangeCallback for NetworkManagerExtStatus {}