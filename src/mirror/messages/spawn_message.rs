#![allow(dead_code)]
use crate::macro_namespace::*;
use crate::macro_network_message::*;
use crate::mirror::messages::message::{MessageDeserializer, MessageSerializer};
use crate::mirror::stable_hash::StableHash;
use crate::mirror::NetworkReader;
use crate::mirror::NetworkWriter;
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