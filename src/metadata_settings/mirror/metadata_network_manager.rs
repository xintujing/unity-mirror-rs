use crate::metadata_settings::unity::metadata_asset::MetadataAsset;
use crate::metadata_settings::unity::metadata_transform::MetadataTransform;
use serde::Deserialize;
use serde_repr::Deserialize_repr;
use std::collections::HashMap;
use unity_mirror_macro::{namespace, MetadataSettingsWrapper};

#[derive(Deserialize_repr, Clone)]
#[repr(u8)]
pub enum ConnectionQualityMethod {
    Simple = 0,     // simple estimation based on rtt and jitter
    Pragmatic = 1,   // based on snapshot interpolation adjustment
}

#[derive(Deserialize_repr, Clone)]
#[repr(u8)]
pub enum HeadlessStartOptions {
    DoNothing = 0,
    AutoStartServer = 1,
    AutoStartClient = 2,
}

#[derive(Deserialize_repr, Clone)]
#[repr(u8)]
pub enum PlayerSpawnMethod {
    Random = 0,
    RoundRobin = 1,
}

#[derive(Deserialize, Clone)]
pub struct MetadataSnapshotSettings {
    #[serde(rename = "bufferTimeMultiplier")]
    pub buffer_time_multiplier: f64,
    #[serde(rename = "bufferLimit")]
    pub buffer_limit: i32,
    #[serde(rename = "catchupNegativeThreshold")]
    pub catchup_negative_threshold: f32,
    #[serde(rename = "catchupPositiveThreshold")]
    pub catchup_positive_threshold: f32,
    #[serde(rename = "catchupSpeed")]
    pub catchup_speed: f64,
    #[serde(rename = "slowdownSpeed")]
    pub slowdown_speed: f64,
    #[serde(rename = "driftEmaDuration")]
    pub drift_ema_duration: i32,
    #[serde(rename = "dynamicAdjustment")]
    pub dynamic_adjustment: bool,
    #[serde(rename = "dynamicAdjustmentTolerance")]
    pub dynamic_adjustment_tolerance: f32,
    #[serde(rename = "deliveryTimeEmaDuration")]
    pub delivery_time_ema_duration: i32,
}

#[allow(unused)]
#[namespace(prefix = "Mirror", rename = "NetworkManager")]
#[derive(Deserialize, MetadataSettingsWrapper, Clone)]
pub struct MetadataNetworkManager {
    #[serde(rename = "dontDestroyOnLoad")]
    pub dont_destroy_on_load: bool,
    // #[serde(rename = "runInBackground")]
    // pub run_in_background: bool,
    // #[serde(rename = "headlessStartMode")]
    // pub headless_start_mode: HeadlessStartOptions,
    #[serde(rename = "editorAutoStart")]
    pub editor_auto_start: bool,
    #[serde(rename = "sendRate")]
    pub send_rate: i32,
    #[serde(rename = "startScene")]
    pub start_scene: MetadataAsset,
    #[serde(rename = "offlineScene")]
    pub offline_scene: Option<MetadataAsset>,
    #[serde(rename = "onlineScene")]
    pub online_scene: Option<MetadataAsset>,
    #[serde(rename = "offlineSceneLoadDelay")]
    pub offline_scene_load_delay: f32,
    #[serde(rename = "playerPrefab")]
    pub player_prefab: MetadataAsset,
    #[serde(rename = "autoCreatePlayer")]
    pub auto_create_player: bool,
    #[serde(rename = "playerSpawnMethod")]
    pub player_spawn_method: PlayerSpawnMethod,
    #[serde(rename = "exceptionsDisconnect")]
    pub exceptions_disconnect: bool,
    #[serde(rename = "snapshotSettings")]
    pub snapshot_settings: MetadataSnapshotSettings,
    #[serde(rename = "evaluationMethod")]
    pub evaluation_method: ConnectionQualityMethod,
    #[serde(rename = "evaluationInterval")]
    pub evaluation_interval: f32,
    #[serde(rename = "timeInterpolationGui")]
    pub time_interpolation_gui: bool,
    #[serde(rename = "spawnPrefabs")]
    pub spawn_prefabs: Vec<MetadataAsset>,
    #[serde(rename = "startPositions")]
    pub start_positions: HashMap<String, Vec<MetadataTransform>>,
}
