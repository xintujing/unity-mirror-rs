use crate::commons::Object;
use crate::metadata_settings::capsule_collider::MetadataCapsuleCollider;
use crate::unity_engine::mono_behaviour::MonoBehaviour;
use crate::namespace;

#[namespace(prefix = "UnityEngine")]
pub struct CapsuleCollider {
    // parent: Collider,
}

impl MonoBehaviour for CapsuleCollider {}

impl CapsuleCollider {
    fn instance(settings: &MetadataCapsuleCollider) -> Self {
        Self {}
    }
}
