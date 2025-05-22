use crate::mirror::messages::message::{MessageDeserializer, MessageSerializer, OnMessageHandler};
use crate::mirror::namespace::Namespace;
use crate::mirror::network_reader::NetworkReader;
use crate::mirror::network_writer::NetworkWriter;
use crate::mirror::stable_hash::StableHash;
use dda_macro::namespace;

#[namespace(prefix = "Mirror")]
#[derive(Debug, PartialEq, Clone, Default)]
pub struct ObjectDestroyMessage {
    pub net_id: u32,
}

impl ObjectDestroyMessage {
    #[allow(unused)]
    pub fn new(net_id: u32) -> ObjectDestroyMessage {
        ObjectDestroyMessage { net_id }
    }
}

impl MessageSerializer for ObjectDestroyMessage {
    fn serialize(&mut self, writer: &mut NetworkWriter)
    where
        Self: Sized,
    {
        writer.write_blittable(Self::get_full_path().hash16());
        writer.write_var_uint(self.net_id);
    }
}

impl MessageDeserializer for ObjectDestroyMessage {
    fn deserialize(reader: &mut NetworkReader) -> Self
    where
        Self: Sized,
    {
        let net_id = reader.read_var_uint();
        Self { net_id }
    }
}

impl OnMessageHandler for ObjectDestroyMessage {}
