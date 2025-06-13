#![allow(dead_code)]
use crate::metadata_settings::mirror::network_behaviours::metadata_network_behaviour::MetadataNetworkBehaviourWrapper;
use serde::Deserialize;
use serde_repr::Deserialize_repr;
use crate::{namespace, settings_wrapper_register};
use crate::commons::Object;
use crate::metadata_settings::Settings;

#[namespace(prefix = "Mirror", rename = "NetworkAnimator")]
#[derive(Deserialize, Clone)]
pub struct MetadataNetworkAnimator {
    #[serde(rename = "clientAuthority")]
    pub client_authority: bool,
    #[serde(rename = "animator")]
    pub animator: MetadataAnimator,
}
settings_wrapper_register!(MetadataNetworkAnimator as MetadataNetworkBehaviourWrapper);

#[derive(Deserialize, Clone)]
pub struct MetadataLayer {
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "fullPathHash")]
    pub full_path_hash: i32,
    #[serde(rename = "normalizedTime")]
    pub normalized_time: f32,
    #[serde(rename = "weight")]
    pub weight: f32,
}

#[derive(Deserialize_repr, Debug, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum MetadataParameterType {
    Float = 1,
    Int = 3,
    Bool = 4,
    Trigger = 9,
}

#[derive(Deserialize, Clone)]
pub struct MetadataParameter {
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "type")]
    pub r#type: MetadataParameterType,
    #[serde(default)]
    pub value: Vec<u8>,
}

#[derive(Deserialize, Clone)]
pub struct MetadataAnimator {
    #[serde(rename = "layers")]
    pub layers: Vec<MetadataLayer>,
    #[serde(rename = "parameters")]
    pub parameters: Vec<MetadataParameter>,
}
