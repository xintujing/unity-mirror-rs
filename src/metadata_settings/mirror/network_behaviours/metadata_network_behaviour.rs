use serde::Deserialize;
use serde_repr::Deserialize_repr;
use unity_mirror_macro::{namespace, MetadataSettingsWrapper};

#[derive(Deserialize_repr, Debug, Clone)]
#[repr(u8)]
pub enum MetadataSyncMode {
    Observers = 0,
    Owner = 1,
}

#[derive(Deserialize_repr, Debug, Clone)]
#[repr(u8)]
pub enum MetadataSyncDirection {
    ServerToClient = 0,
    ClientToServer = 1,
}

#[namespace("Mirror", rename = "NetworkBehaviour")]
#[derive(Deserialize, MetadataSettingsWrapper, Clone)]
pub struct MetadataNetworkBehaviour {
    #[serde(rename = "syncMode")]
    pub sync_mode: MetadataSyncMode,
    #[serde(rename = "syncDirection")]
    pub sync_direction: MetadataSyncDirection,
    #[serde(rename = "syncInterval")]
    pub sync_interval: f32,
}
