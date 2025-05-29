use crate::commons::object::Object;
use crate::mirror::messages::message::{MessageDeserializer, MessageSerializer};
use crate::mirror::network_reader::NetworkReader;
use crate::mirror::network_writer::NetworkWriter;
use crate::mirror::stable_hash::StableHash;
use unity_mirror_macro::{namespace, Message};

#[namespace(prefix = "Mirror")]
#[derive(Debug, PartialEq, Clone, Default, Message)]
pub struct EntityStateMessage {
    pub net_id: u32,
    pub payload: Vec<u8>,
}

impl EntityStateMessage {
    #[allow(unused)]
    pub fn new(net_id: u32, payload: Vec<u8>) -> EntityStateMessage {
        Self { net_id, payload }
    }

    #[allow(unused)]
    pub fn get_payload_content(&self) -> Vec<u8> {
        self.payload[4..].to_vec()
    }
}

impl MessageSerializer for EntityStateMessage {
    fn serialize(&mut self, writer: &mut NetworkWriter)
    where
        Self: Sized,
    {
        writer.write_blittable(Self::get_full_name().hash16());
        writer.write_blittable_compress(self.net_id);
        writer.write_slice_and_size(self.payload.as_slice());
    }
}

impl MessageDeserializer for EntityStateMessage {
    fn deserialize(reader: &mut NetworkReader) -> Self
    where
        Self: Sized,
    {
        let net_id = reader.read_blittable_compress();
        let payload = reader.read_slice_and_size();
        Self {
            net_id,
            payload: payload.to_vec(),
        }
    }
}
