#![allow(dead_code)]
use crate::macro_namespace::*;
use crate::mirror::messages::message::{MessageDeserializer, MessageSerializer};
use crate::mirror::stable_hash::StableHash;
use crate::mirror::NetworkReader;
use crate::mirror::NetworkWriter;
use crate::macro_network_message::*;
use nalgebra::{Quaternion, Vector3};

#[derive(Clone, Debug, Default, PartialEq, Copy)]
#[repr(u8)]
pub enum AuthorityFlags {
    #[default]
    None = 0,
    IsOwner = 1,
    IsLocalPlayer = 2,
}

#[namespace(prefix = "Mirror")]
#[derive(Debug, PartialEq, Clone, Default, NetworkMessage)]
pub struct SpawnMessage {
    pub net_id: u32,
    authority_flags: u8,
    pub scene_id: u64,
    pub asset_id: u32,
    pub position: Vector3<f32>,
    pub rotation: Quaternion<f32>,
    pub scale: Vector3<f32>,
    pub payload: Vec<u8>,
}

impl SpawnMessage {
    pub fn new(
        net_id: u32,
        is_local_player: bool,
        is_owner: bool,
        scene_id: u64,
        asset_id: u32,
        position: Vector3<f32>,
        rotation: Quaternion<f32>,
        scale: Vector3<f32>,
        payload: Vec<u8>,
    ) -> SpawnMessage {
        let mut message = SpawnMessage {
            net_id,
            scene_id,
            asset_id,
            position,
            rotation,
            scale,
            payload,
            authority_flags: 0,
        };

        message.set_is_local_player(is_local_player);
        message.set_is_owner(is_owner);

        message
    }

    // 检查是否具有某个标志
    pub fn has_flag(&self, flag: AuthorityFlags) -> bool {
        self.authority_flags & (flag as u8) != 0
    }
    // 设置某个标志
    pub fn set_flag(&mut self, flag: AuthorityFlags, value: bool) {
        if value {
            self.authority_flags |= flag as u8; // 设置标志
        } else {
            self.authority_flags &= !(flag as u8); // 清除标志
        }
    }
    // 方便性 getter: 检查 IsOwner
    pub fn is_owner(&self) -> bool {
        self.has_flag(AuthorityFlags::IsOwner)
    }
    // 方便性 setter: 设置 IsOwner
    pub fn set_is_owner(&mut self, value: bool) {
        self.set_flag(AuthorityFlags::IsOwner, value);
    }
    // 方便性 getter: 检查 IsLocalPlayer
    pub fn is_local_player(&self) -> bool {
        self.has_flag(AuthorityFlags::IsLocalPlayer)
    }
    // 方便性 setter: 设置 IsLocalPlayer
    pub fn set_is_local_player(&mut self, value: bool) {
        self.set_flag(AuthorityFlags::IsLocalPlayer, value);
    }
}

impl MessageSerializer for SpawnMessage {
    fn serialize(&mut self, writer: &mut NetworkWriter)
    where
        Self: Sized,
    {
        writer.write_blittable(Self::get_full_name().hash16());
        writer.write_blittable_compress(self.net_id);
        writer.write_blittable(self.authority_flags);
        writer.write_blittable_compress(self.scene_id);
        writer.write_blittable_compress(self.asset_id);
        writer.write_blittable(self.position);
        writer.write_blittable(self.rotation);
        writer.write_blittable(self.scale);
        writer.write_slice_and_size(self.payload.as_slice());
    }
}

impl MessageDeserializer for SpawnMessage {
    fn deserialize(reader: &mut NetworkReader) -> Self
    where
        Self: Sized,
    {
        let net_id = reader.read_blittable_compress();
        let authority_flags = reader.read_blittable();
        let scene_id = reader.read_blittable_compress();
        let asset_id = reader.read_blittable_compress();
        let position = reader.read_blittable();
        let rotation = reader.read_blittable();
        let scale = reader.read_blittable();
        let payload = reader.read_slice_and_size();
        Self {
            net_id,
            authority_flags,
            scene_id,
            asset_id,
            position,
            rotation,
            scale,
            payload: payload.to_vec(),
        }
    }
}
