use crate::metadata_settings::mirror::network_behaviours::metadata_network_behaviour::MetadataNetworkBehaviourWrapper;
use serde::Deserialize;
use crate::{namespace, settings_wrapper_register};
use crate::commons::Object;
use crate::metadata_settings::Settings;

#[namespace(prefix = "Mirror", rename = "NetworkTransformReliable")]
#[derive(Deserialize, Debug, Clone)]
pub struct MetadataNetworkTransformReliable {}
settings_wrapper_register!(MetadataNetworkTransformReliable as MetadataNetworkBehaviourWrapper);
