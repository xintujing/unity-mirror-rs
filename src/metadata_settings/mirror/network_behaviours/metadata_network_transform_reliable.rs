use crate::commons::Object;
use crate::macro_namespace::*;
use crate::metadata_settings::mirror::network_behaviours::metadata_network_behaviour::MetadataNetworkBehaviourWrapper;
use crate::metadata_settings::Settings;
use crate::settings_wrapper_register;
use serde::Deserialize;

#[namespace(prefix = "Mirror", rename = "NetworkTransformReliable")]
#[derive(Deserialize, Debug, Clone)]
pub struct MetadataNetworkTransformReliable {}
settings_wrapper_register!(MetadataNetworkTransformReliable as MetadataNetworkBehaviourWrapper);
