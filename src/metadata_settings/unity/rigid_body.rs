#![allow(dead_code)]
use crate::commons::object::Object;
use crate::metadata_settings::unity::collider::LayerMask;
use crate::metadata_settings::unity::metadata_component::MetadataComponentWrapper;
use serde::Deserialize;
use unity_mirror_macro_rs::{namespace, settings_wrapper_register, MetadataSettingsWrapper};

#[namespace(prefix = "UnityEngine", rename = "Rigidbody")]
#[derive(Deserialize, MetadataSettingsWrapper, Clone)]
pub struct MetadataRigidBody {
    #[serde(rename = "instanceId")]
    pub instance_id: i32,
    pub mass: f32,
    pub drag: f32,
    #[serde(rename = "angularDrag")]
    pub angular_drag: f32,
    #[serde(rename = "automaticCenterOfMass")]
    pub automatic_center_of_mass: bool,
    #[serde(rename = "automaticInertiaTensor")]
    pub automatic_inertia_tensor: bool,
    #[serde(rename = "useGravity")]
    pub use_gravity: bool,
    #[serde(rename = "isKinematic")]
    pub is_kinematic: bool,
    pub interpolation: i32,
    #[serde(rename = "collisionDetectionMode")]
    pub collision_detection_mode: i32,
    pub constraints: i32,
    #[serde(rename = "includeLayers")]
    pub include_layers: LayerMask,
    #[serde(rename = "excludeLayers")]
    pub exclude_layers: LayerMask,
}

impl Object for MetadataRigidBodyWrapper {
    fn get_full_name() -> &'static str
    where
        Self: Sized,
    {
        "UnityEngine.Rigidbody"
    }
}

settings_wrapper_register!(MetadataRigidBodyWrapper as MetadataComponentWrapper);
