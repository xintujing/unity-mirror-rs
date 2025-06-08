use crate::metadata_settings::mirror::network_behaviours::metadata_network_behaviour::MetadataNetworkBehaviourWrapper;
use serde::Deserialize;
use unity_mirror_macro_rs::{namespace, settings_wrapper_register};

#[namespace(prefix = "Mirror", rename = "NetworkTransformUnreliable")]
#[derive(Deserialize, Debug, Clone)]
pub struct MetadataNetworkTransformUnreliable {
    #[serde(rename = "bufferResetMultiplier")]
    pub buffer_reset_multiplier: f32,
    #[serde(rename = "positionSensitivity")]
    pub position_sensitivity: f32,
    #[serde(rename = "rotationSensitivity")]
    pub rotation_sensitivity: f32,
    #[serde(rename = "scaleSensitivity")]
    pub scale_sensitivity: f32,
}
settings_wrapper_register!(MetadataNetworkTransformUnreliable as MetadataNetworkBehaviourWrapper);
