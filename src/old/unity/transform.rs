use crate::commons::reference::Reference;
use crate::unity::game_object::GameObject;
use nalgebra::{Quaternion, Vector3};

#[derive(Default)]
pub struct Transform {
    pub(super) parent: Option<Reference<Transform>>,
    pub(super) children: Vec<Reference<Transform>>,
    pub(super) game_object: Reference<GameObject>,
    pub(super) position: Vector3<f32>,
    pub(super) local_position: Vector3<f32>,
    pub(super) rotation: Quaternion<f32>,
    pub(super) local_rotation: Quaternion<f32>,
    pub(super) local_scale: Vector3<f32>,
}

impl Transform {
    pub fn get_position(&self) -> Vector3<f32> {
        self.position
    }

    pub fn set_position(&mut self, position: Vector3<f32>) {
        self.position = position;
    }

    pub fn get_local_position(&self) -> Vector3<f32> {
        self.local_position
    }

    pub fn set_local_position(&mut self, local_position: Vector3<f32>) {
        self.local_position = local_position;
    }

    pub fn get_rotation(&self) -> Quaternion<f32> {
        self.rotation
    }

    pub fn set_rotation(&mut self, rotation: Quaternion<f32>) {
        self.rotation = rotation;
    }

    pub fn get_local_rotation(&self) -> Quaternion<f32> {
        self.local_rotation
    }

    pub fn set_local_rotation(&mut self, local_rotation: Quaternion<f32>) {
        self.local_rotation = local_rotation;
    }

    pub fn get_local_scale(&self) -> Vector3<f32> {
        self.local_scale
    }

    pub fn set_local_scale(&mut self, local_scale: Vector3<f32>) {
        self.local_scale = local_scale;
    }

    pub fn get_parent(&self) -> Option<Reference<Transform>> {
        self.parent.clone()
    }

    pub fn children_len(&self) -> usize {
        self.children.len()
    }

    pub fn get_child(&self, index: usize) -> Option<Reference<Transform>> {
        if index >= self.children.len() {
            return None;
        }
        self.children.get(index).cloned()
    }

    pub fn get_game_object(&self) -> Reference<GameObject> {
        self.game_object.clone()
    }
}
