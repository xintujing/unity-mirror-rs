#![allow(dead_code)]
use crate::metadata_settings::mirror::network_behaviours::metadata_network_behaviour::MetadataNetworkBehaviourWrapper;
use serde::Deserialize;
use serde_repr::Deserialize_repr;
use unity_mirror_macro::{namespace, settings_wrapper_register};

#[namespace("Mirror", rename = "NetworkAnimator")]
#[derive(Deserialize, Clone)]
pub(crate) struct MetadataNetworkAnimator {
    #[serde(rename = "clientAuthority")]
    pub(crate) client_authority: bool,
    #[serde(rename = "animator")]
    pub(crate) animator: MetadataAnimator,
}
settings_wrapper_register!(MetadataNetworkAnimator as MetadataNetworkBehaviourWrapper);

#[derive(Deserialize, Clone)]
pub(crate) struct MetadataLayer {
    #[serde(rename = "name")]
    pub(crate) name: String,
    #[serde(rename = "fullPathHash")]
    pub(crate) full_path_hash: i32,
    #[serde(rename = "normalizedTime")]
    pub(crate) normalized_time: f32,
    #[serde(rename = "weight")]
    pub(crate) weight: f32,
}

#[derive(Deserialize_repr, Debug, Clone, Eq, PartialEq)]
#[repr(u8)]
pub(crate) enum MetadataParameterType {
    Float = 1,
    Int = 3,
    Bool = 4,
    Trigger = 9,
}

#[derive(Deserialize, Clone)]
pub(crate) struct MetadataParameter {
    #[serde(rename = "name")]
    pub(crate) name: String,
    #[serde(rename = "type")]
    pub(crate) r#type: MetadataParameterType,
    #[serde(default)]
    pub(crate) value: Vec<u8>,
}

#[derive(Deserialize, Clone)]
pub(crate) struct MetadataAnimator {
    #[serde(rename = "layers")]
    pub(crate) layers: Vec<MetadataLayer>,
    #[serde(rename = "parameters")]
    pub(crate) parameters: Vec<MetadataParameter>,
}
