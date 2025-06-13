use crate::mirror::messages::message::{MessageDeserializer, MessageSerializer};
use crate::mirror::stable_hash::StableHash;
use crate::mirror::NetworkReader;
use crate::mirror::NetworkWriter;
use crate::macro_namespace::*;
use crate::macro_network_message::*;

#[namespace(prefix = "Mirror")]
#[derive(Debug, PartialEq, Clone, Default, NetworkMessage)]
pub struct AddPlayerMessage;

impl AddPlayerMessage {
    #[allow(unused)]
    fn new() -> Self {
        Self
    }
}

impl MessageSerializer for AddPlayerMessage {
    fn serialize(&mut self, writer: &mut NetworkWriter)
    where
        Self: Sized,
    {
        writer.write_blittable(Self::get_full_name().hash16());
    }
}

impl MessageDeserializer for AddPlayerMessage {
    fn deserialize(reader: &mut NetworkReader) -> Self
    where
        Self: Sized,
    {
        let _ = reader;
        Self
    }
}
