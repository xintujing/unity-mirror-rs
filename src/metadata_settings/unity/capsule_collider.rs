use crate::metadata_settings::unity::collider::MetadataColliderWrapper;
use serde::Deserialize;
use unity_mirror_rs_macro::{namespace, settings_wrapper_register};

#[namespace("UnityEngine", rename = "CapsuleCollider")]
#[derive(Deserialize)]
pub struct MetadataCapsuleCollider {
    pub center: [f32; 3],
    pub radius: f32,
    pub height: f32,
    pub direction: i32,
}

settings_wrapper_register!(MetadataCapsuleCollider as MetadataColliderWrapper);
