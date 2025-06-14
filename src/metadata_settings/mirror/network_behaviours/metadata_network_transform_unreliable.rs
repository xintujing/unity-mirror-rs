use crate::commons::Object;
use crate::macro_namespace::*;
use crate::metadata_settings::mirror::network_behaviours::metadata_network_behaviour::MetadataNetworkBehaviourWrapper;
use crate::metadata_settings::Settings;
use crate::settings_wrapper_register;
use serde::Deserialize;

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
