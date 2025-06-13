use crate::metadata_settings::unity::collider::MetadataColliderWrapper;
use serde::Deserialize;
use crate::{namespace, settings_wrapper_register};
use crate::commons::Object;
use crate::metadata_settings::Settings;

#[namespace(prefix = "UnityEngine", rename = "CapsuleCollider")]
#[derive(Deserialize, Clone)]
pub struct MetadataCapsuleCollider {
    pub center: [f32; 3],
    pub radius: f32,
    pub height: f32,
    pub direction: i32,
}

settings_wrapper_register!(MetadataCapsuleCollider as MetadataColliderWrapper);
