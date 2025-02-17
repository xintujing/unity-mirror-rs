use crate::mirror::core::network_reader::NetworkReader;
use crate::mirror::core::network_writer::NetworkWriter;
use crate::mirror::core::tools::stable_hash::StableHash;
use std::any::Any;
use unity_mirror_rs_macro::NetworkMessage;

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

#[derive(Debug, PartialEq, Clone, Default, NetworkMessage)]
pub struct CommandMessage {
    pub net_id: u32,
    pub component_index: u8,
    pub function_hash: u16,
    pub payload: Vec<u8>,
}

impl NetworkMessageTrait for CommandMessage {
    fn get_full_name() -> &'static str {
        "CommandMessage"
    }
}
