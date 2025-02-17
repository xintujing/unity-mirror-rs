use crate::mirror::core::network_reader::NetworkReader;
use crate::mirror::core::network_writer::NetworkWriter;
use crate::mirror::core::tools::stable_hash::StableHash;
use nalgebra::{Quaternion, Vector3};
use std::any::Any;
use unity_mirror_rs_macro::NetworkMessage;

/// TimeSnapshotMessage
#[derive(Debug, PartialEq, Clone, Copy, Default, NetworkMessage)]
pub struct TimeSnapshotMessage;
impl NetworkMessageTrait for TimeSnapshotMessage {
    fn get_full_name() -> &'static str {
        "Mirror.TimeSnapshotMessage"
    }
}

/// ReadyMessage
#[derive(Debug, PartialEq, Clone, Copy, Default, NetworkMessage)]
pub struct ReadyMessage;
impl NetworkMessageTrait for ReadyMessage {
    fn get_full_name() -> &'static str {
        "Mirror.ReadyMessage"
    }
}

/// NotReadyMessage
#[derive(Debug, PartialEq, Clone, Copy, Default, NetworkMessage)]
pub struct NotReadyMessage;
impl NetworkMessageTrait for NotReadyMessage {
    fn get_full_name() -> &'static str {
        "Mirror.NotReadyMessage"
    }
}

/// AddPlayerMessage
#[derive(Debug, PartialEq, Clone, Copy, Default, NetworkMessage)]
pub struct AddPlayerMessage;
impl NetworkMessageTrait for AddPlayerMessage {
    fn get_full_name() -> &'static str {
        "Mirror.AddPlayerMessage"
    }
}

/// SceneMessage
#[derive(Debug, PartialEq, Clone, Copy, Default)]
#[repr(u8)]
pub enum SceneOperation {
    #[default]
    Normal = 0,
    LoadAdditive = 1,
    UnloadAdditive = 2,
}
impl SceneOperation {
    pub fn from(value: u8) -> SceneOperation {
        match value {
            0 => SceneOperation::Normal,
            1 => SceneOperation::LoadAdditive,
            2 => SceneOperation::UnloadAdditive,
            _ => SceneOperation::Normal,
        }
    }
    pub fn to_u8(&self) -> u8 {
        *self as u8
    }
}
#[derive(Debug, PartialEq, Clone, Default, NetworkMessage)]
pub struct SceneMessage {
    pub scene_name: String,
    pub operation: u8,
    pub custom_handling: bool,
}
impl NetworkMessageTrait for SceneMessage {
    fn get_full_name() -> &'static str {
        "Mirror.SceneMessage"
    }
}

#[derive(Debug, PartialEq, Clone, Default, NetworkMessage)]
pub struct CommandMessage {
    pub net_id: u32,
    pub component_index: u8,
    pub function_hash: u16,
    pub payload: Vec<u8>,
}

impl NetworkMessageTrait for CommandMessage {
    fn get_full_name() -> &'static str {
        "Mirror.CommandMessage"
    }
}

/// RpcMessage
#[derive(Debug, PartialEq, Clone, Default, NetworkMessage)]
pub struct RpcMessage {
    pub net_id: u32,
    pub component_index: u8,
    pub function_hash: u16,
    pub payload: Vec<u8>,
}
impl NetworkMessageTrait for RpcMessage {
    fn get_full_name() -> &'static str {
        "Mirror.RpcMessage"
    }
}

/// SpawnMessage
#[derive(Debug, PartialEq, Clone, Default, NetworkMessage)]
pub struct SpawnMessage {
    pub net_id: u32,
    pub is_local_player: bool,
    pub is_owner: bool,
    pub scene_id: u64,
    pub asset_id: u32,
    pub position: Vector3<f32>,
    pub rotation: Quaternion<f32>,
    pub scale: Vector3<f32>,
    pub payload: Vec<u8>,
}
impl NetworkMessageTrait for SpawnMessage {
    fn get_full_name() -> &'static str {
        "Mirror.SpawnMessage"
    }
}

/// ObjectDestroyMessage
#[derive(Debug, PartialEq, Clone, Default, NetworkMessage)]
pub struct ChangeOwnerMessage {
    pub net_id: u32,
    pub is_owner: bool,
    pub is_local_player: bool,
}
impl NetworkMessageTrait for ChangeOwnerMessage {
    fn get_full_name() -> &'static str {
        "Mirror.ChangeOwnerMessage"
    }
}

/// ObjectDestroyMessage
#[derive(Debug, PartialEq, Clone, Default, NetworkMessage)]
pub struct ObjectSpawnStartedMessage;
impl NetworkMessageTrait for ObjectSpawnStartedMessage {
    fn get_full_name() -> &'static str {
        "Mirror.ObjectSpawnStartedMessage"
    }
}

#[derive(Debug, PartialEq, Clone, Default, NetworkMessage)]
pub struct ObjectSpawnFinishedMessage;

impl NetworkMessageTrait for ObjectSpawnFinishedMessage {
    fn get_full_name() -> &'static str {
        "Mirror.ObjectSpawnFinishedMessage"
    }
}

/// ObjectDestroyMessage
#[derive(Debug, PartialEq, Clone, Default, NetworkMessage)]
pub struct ObjectDestroyMessage {
    pub net_id: u32,
}
impl NetworkMessageTrait for ObjectDestroyMessage {
    fn get_full_name() -> &'static str {
        "Mirror.ObjectDestroyMessage"
    }
}

/// ObjectHideMessage
#[derive(Debug, PartialEq, Clone, Default, NetworkMessage)]
pub struct ObjectHideMessage {
    pub net_id: u32,
}
impl NetworkMessageTrait for ObjectHideMessage {
    fn get_full_name() -> &'static str {
        "Mirror.ObjectHideMessage"
    }
}

/// ObjectDestroyMessage
#[derive(Debug, PartialEq, Clone, Default, NetworkMessage)]
pub struct EntityStateMessage {
    pub net_id: u32,
    pub payload: Vec<u8>,
}
impl NetworkMessageTrait for EntityStateMessage {
    fn get_full_name() -> &'static str {
        "Mirror.EntityStateMessage"
    }
}

/// NetworkPingMessage
#[derive(Debug, PartialEq, Clone, Default, NetworkMessage)]
pub struct NetworkPingMessage {
    pub local_time: f64,
    pub predicted_time_adjusted: f64,
}
impl NetworkMessageTrait for NetworkPingMessage {
    fn get_full_name() -> &'static str {
        "Mirror.NetworkPingMessage"
    }
}

/// NetworkPongMessage
#[derive(Debug, PartialEq, Clone, Default, NetworkMessage)]
pub struct NetworkPongMessage {
    pub local_time: f64,
    pub prediction_error_unadjusted: f64,
    pub prediction_error_adjusted: f64,
}
impl NetworkMessageTrait for NetworkPongMessage {
    fn get_full_name() -> &'static str {
        "Mirror.NetworkPongMessage"
    }
}

pub trait NetworkMessagePreTrait: Default {
    fn serialize(&mut self, writer: &mut NetworkWriter);
    fn deserialize(reader: &mut NetworkReader) -> Self
    where
        Self: Sized;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub trait NetworkMessageTrait: Send + Sync + NetworkMessagePreTrait {
    fn get_hash_code() -> u16
    where
        Self: Sized,
    {
        Self::get_full_name().get_stable_hash_code16()
    }
    fn get_full_name() -> &'static str
    where
        Self: Sized;
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::mirror::core::network_writer_pool::NetworkWriterPool;

    #[test]
    fn test_network_message_trait() {
        let mut writer = NetworkWriterPool::get();
        let mut message = NetworkPingMessage {
            local_time: 1.0,
            predicted_time_adjusted: 2.0,
        };
        message.serialize(&mut writer);

        println!("{:?}", writer.to_bytes());
    }
}
