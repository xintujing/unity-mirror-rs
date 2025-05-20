use crate::metadata_settings::unity::capsule_collider::MetadataCapsuleCollider;
use crate::unity_engine::mono_behaviour::MonoBehaviour;
use unity_mirror_macro::namespace;

#[namespace("UnityEngine")]
pub struct CapsuleCollider {
    // parent: Collider,
}

impl MonoBehaviour for CapsuleCollider {}

impl CapsuleCollider {
    fn instance(settings: &MetadataCapsuleCollider) -> Self {
        Self {}
    }
}
