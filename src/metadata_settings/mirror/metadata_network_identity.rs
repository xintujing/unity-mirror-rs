use crate::commons::object::Object;
use crate::metadata_settings::mirror::network_behaviours::metadata_network_behaviour::MetadataNetworkBehaviourWrapper;
use crate::metadata_settings::unity::metadata_component::MetadataComponentWrapper;
use serde::Deserialize;
use serde_repr::Deserialize_repr;
use unity_mirror_macro::{namespace, settings_wrapper_register, MetadataSettingsWrapper};

#[namespace(prefix = "Mirror", rename = "NetworkIdentity")]
#[derive(Deserialize, MetadataSettingsWrapper)]
#[derive(Clone)]
pub struct MetadataNetworkIdentity {
    #[serde(rename = "assetId")]
    pub asset_id: u32,
    #[serde(rename = "sceneId")]
    pub scene_id: String,
    #[serde(rename = "serverOnly")]
    pub server_only: bool,
    #[serde(rename = "visibility")]
    pub visibility: MetadataVisibility,
    #[serde(rename = "networkBehaviours")]
    pub network_behaviours: Vec<MetadataNetworkBehaviourWrapper>,
}

#[derive(Deserialize_repr, Clone, Copy)]
#[repr(u8)]
pub enum MetadataVisibility {
    Default = 0,
    ForceHidden = 1,
    ForceShown = 2,
}

impl crate::commons::object::Object for MetadataNetworkIdentityWrapper {
    fn get_full_name() -> &'static str
    where
        Self: Sized,
    {
        "Mirror.NetworkIdentity"
    }
}

settings_wrapper_register!(MetadataNetworkIdentityWrapper as MetadataComponentWrapper);
