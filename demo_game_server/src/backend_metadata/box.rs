use serde::Deserialize;
use unity_mirror_rs::metadata_settings::mirror::network_behaviours::metadata_network_behaviour::MetadataNetworkBehaviourWrapper;
// use unity_mirror_rs::unity_mirror_macro_rs::{namespace, settings_wrapper_register};

#[namespace(rename = "Box")]
#[derive(Deserialize, Clone)]
pub struct MetadataBox {}
settings_wrapper_register!(MetadataBox as MetadataNetworkBehaviourWrapper);
