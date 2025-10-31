use serde::Deserialize;
use unity_mirror_rs::macro_namespace::*;
use unity_mirror_rs::metadata_settings::metadata_asset::MetadataAsset;
use unity_mirror_rs::metadata_settings::MetadataNetworkBehaviourWrapper;
use unity_mirror_rs::metadata_settings::Settings;
use unity_mirror_rs::settings_wrapper_register;

#[namespace(prefix = "DaDaA.Scripts", rename = "NetworkManagerExtStatus")]
#[derive(Deserialize, Clone, Default)]
pub struct MetadataNetworkManagerExtStatus {
    #[serde(default, rename = "gameObject")]
    pub game_object: MetadataAsset,
}

settings_wrapper_register!(MetadataNetworkManagerExtStatus as MetadataNetworkBehaviourWrapper);
