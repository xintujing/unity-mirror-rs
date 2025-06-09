use crate::metadata_settings::unity::collider::MetadataColliderWrapper;
use serde::Deserialize;
use unity_mirror_macro_rs::{namespace, settings_wrapper_register};

#[namespace(prefix = "UnityEngine", rename = "CapsuleCollider")]
#[derive(Deserialize, Clone)]
pub struct MetadataCapsuleCollider {
    pub center: [f32; 3],
    pub radius: f32,
    pub height: f32,
    pub direction: i32,
}

settings_wrapper_register!(MetadataCapsuleCollider as MetadataColliderWrapper);
