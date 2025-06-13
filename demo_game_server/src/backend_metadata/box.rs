use unity_mirror_rs::metadata_settings::Settings;
use serde::Deserialize;
use unity_mirror_rs::commons::Object;
use unity_mirror_rs::metadata_settings::MetadataNetworkBehaviourWrapper;
use unity_mirror_rs::{namespace, settings_wrapper_register};

#[namespace(rename = "Box")]
#[derive(Deserialize, Clone)]
pub struct MetadataBox {}
settings_wrapper_register!(MetadataBox as MetadataNetworkBehaviourWrapper);
