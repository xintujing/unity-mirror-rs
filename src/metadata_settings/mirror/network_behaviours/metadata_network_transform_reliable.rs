use crate::metadata_settings::mirror::network_behaviours::metadata_network_behaviour::MetadataNetworkBehaviourWrapper;
use serde::Deserialize;
use unity_mirror_rs_macro::{namespace, settings_wrapper_register};

#[namespace("Mirror", rename = "NetworkTransformReliable")]
#[derive(Deserialize, Debug, Clone)]
pub struct MetadataNetworkTransformReliable {}
settings_wrapper_register!(MetadataNetworkTransformReliable as MetadataNetworkBehaviourWrapper);
