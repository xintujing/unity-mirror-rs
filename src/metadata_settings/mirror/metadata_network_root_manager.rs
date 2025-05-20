use crate::metadata_settings::mirror::metadata_network_manager::MetadataNetworkManagerWrapper;
use crate::metadata_settings::unity::metadata_asset::MetadataAsset;
use serde::Deserialize;
use unity_mirror_macro::{namespace, settings_wrapper_register};

#[namespace("Mirror", rename = "NetworkRoomManager")]
#[derive(Deserialize, Clone)]
pub struct MetadataNetworkRootManager {
    #[serde(rename = "minPlayers")]
    pub min_players: i32,
    // 预制可用于房间播放器
    #[serde(rename = "roomPlayerPrefab")]
    pub room_player_prefab: MetadataAsset,
    // 用于房间的场景 这类似于 NetworkRoomManager 的 offline scene
    #[serde(rename = "roomScene")]
    pub room_scene: String,
    // 从房间玩游戏的场景 这类似于 NetworkRoomManager 的online scene。
    #[serde(rename = "gameplayScene")]
    pub gameplay_scene: String,

    #[serde(rename = "clientIndex")]
    pub client_index: usize,
}
settings_wrapper_register!(MetadataNetworkRootManager as MetadataNetworkManagerWrapper);
