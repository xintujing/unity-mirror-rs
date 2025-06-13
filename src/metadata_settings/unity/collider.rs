#![allow(dead_code)]
use crate::commons::Object;
use crate::metadata_settings::unity::metadata_component::MetadataComponentWrapper;
use serde::Deserialize;
use crate::{namespace, settings_wrapper_register, MetadataSettingsWrapper};
use crate::metadata_settings::Settings;

#[derive(Deserialize, Clone)]
pub struct LayerMask {
    pub value: i32,
}

#[namespace(prefix = "UnityEngine", rename = "Collider")]
#[derive(Deserialize, MetadataSettingsWrapper, Clone)]
pub struct MetadataCollider {
    #[serde(rename = "instanceId")]
    pub instance_id: i32,
    #[serde(rename = "isTrigger")]
    pub is_trigger: bool,
    #[serde(rename = "providesContacts")]
    pub provides_contacts: bool,
    // pub material: i32,
    #[serde(rename = "layerOverridePriority")]
    pub layer_override_priority: i32,
    #[serde(rename = "includeLayers")]
    pub include_layers: LayerMask,
    #[serde(rename = "excludeLayers")]
    pub exclude_layers: LayerMask,
}

impl Object for MetadataColliderWrapper {
    fn get_full_name() -> &'static str
    where
        Self: Sized,
    {
        "UnityEngine.Collider"
    }
}

settings_wrapper_register!(MetadataColliderWrapper as MetadataComponentWrapper);
