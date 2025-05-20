use crate::commons::revel_weak::RevelWeak;
use crate::metadata_settings::unity::metadata_transform::MetadataTransform;
use crate::unity_engine::GameObject;
use nalgebra::{Quaternion, Vector3};

#[derive(Default, Clone)]
pub struct Transform {
    pub(super) instance_id: i32,
    pub(super) parent: RevelWeak<Transform>,
    pub(super) children: Vec<RevelWeak<Transform>>,
    pub(super) game_object: RevelWeak<GameObject>,
    pub(super) position: Vector3<f32>,
    pub(super) local_position: Vector3<f32>,
    pub(super) rotation: Quaternion<f32>,
    pub(super) local_rotation: Quaternion<f32>,
    pub(super) local_scale: Vector3<f32>,
}

impl Transform {
    pub fn new_with_metadata(metadata: &MetadataTransform) -> Self {
        Transform {
            instance_id: metadata.instance_id,
            parent: RevelWeak::default(),
            children: vec![],
            game_object: RevelWeak::default(),
            position: Vector3::new(
                metadata.position[0],
                metadata.position[1],
                metadata.position[2],
            ),
            rotation: Quaternion::new(
                metadata.rotation[3],
                metadata.rotation[0],
                metadata.rotation[1],
                metadata.rotation[2],
            ),
            local_position: Vector3::new(
                metadata.local_position[0],
                metadata.local_position[1],
                metadata.local_position[2],
            ),
            local_rotation: Quaternion::new(
                metadata.local_rotation[3],
                metadata.local_rotation[0],
                metadata.local_rotation[1],
                metadata.local_rotation[2],
            ),
            local_scale: Vector3::new(
                metadata.local_scale[0],
                metadata.local_scale[1],
                metadata.local_scale[2],
            ),
        }
    }
}
