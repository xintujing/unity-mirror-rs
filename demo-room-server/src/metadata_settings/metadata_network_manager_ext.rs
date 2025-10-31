use crate::metadata_settings::network_manager_ext_status::MetadataNetworkManagerExtStatus;
use serde::Deserialize;
use unity_mirror_rs::macro_namespace::*;
use unity_mirror_rs::metadata_settings::{MetadataNetworkManagerWrapper, Settings};
use unity_mirror_rs::settings_wrapper_register;

#[namespace(prefix = "DaDaA.Scripts", rename = "NetworkManagerExt")]
#[derive(Deserialize, Clone)]
pub struct MetadataNetworkManagerExt {
    #[serde(default, rename = "minPlayers")]
    pub min_players: u32,
    #[serde(default, rename = "networkManagerExtStatus")]
    pub network_manager_ext_status: MetadataNetworkManagerExtStatus,
}

settings_wrapper_register!(MetadataNetworkManagerExt as MetadataNetworkManagerWrapper);
