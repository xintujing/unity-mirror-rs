use serde::Deserialize;
use unity_mirror_rs::macro_namespace::*;
use unity_mirror_rs::metadata_settings::collider::MetadataColliderWrapper;
use unity_mirror_rs::metadata_settings::Settings;
use unity_mirror_rs::settings_wrapper_register;

#[namespace(prefix = "UnityEngine", rename = "CharacterController")]
#[derive(Deserialize, Clone)]
pub struct MetadataCharacterController {}

settings_wrapper_register!(MetadataCharacterController as MetadataColliderWrapper);
