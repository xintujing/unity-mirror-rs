use crate::commons::object::Object;
use crate::mirror::messages::message::{MessageDeserializer, MessageSerializer};
use crate::mirror::network_reader::NetworkReader;
use crate::mirror::network_writer::NetworkWriter;
use crate::mirror::stable_hash::StableHash;
use unity_mirror_macro::{namespace, Message};

#[namespace(prefix = "Mirror")]
#[derive(Debug, PartialEq, Clone, Default, Message)]
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
        writer.write_blittable(Self::get_full_name().hash16());
        writer.write_blittable_compress(self.net_id);
    }
}

impl MessageDeserializer for ObjectDestroyMessage {
    fn deserialize(reader: &mut NetworkReader) -> Self
    where
        Self: Sized,
    {
        let net_id = reader.read_blittable_compress();
        Self { net_id }
    }
}
