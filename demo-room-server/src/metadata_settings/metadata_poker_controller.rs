use serde::Deserialize;
use unity_mirror_rs::macro_namespace::*;
use unity_mirror_rs::metadata_settings::{MetadataNetworkBehaviourWrapper, Settings};
use unity_mirror_rs::settings_wrapper_register;

#[namespace(prefix = "DaDaA.Scripts", rename = "PokerController")]
#[derive(Deserialize, Clone)]
pub struct MetadataPokerController {}

settings_wrapper_register!(MetadataPokerController as MetadataNetworkBehaviourWrapper);
