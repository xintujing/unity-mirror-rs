use crate::commons::object::Object;
use crate::mirror::messages::message::{MessageDeserializer, MessageSerializer};
use crate::mirror::network_reader::NetworkReader;
use crate::mirror::network_writer::NetworkWriter;
use crate::mirror::stable_hash::StableHash;
use unity_mirror_macro::{namespace, NetworkMessage};

#[namespace(prefix = "Mirror")]
#[derive(Debug, PartialEq, Clone, Default, NetworkMessage)]
pub struct RpcMessage {
    pub net_id: u32,
    pub component_index: u8,
    pub function_hash: u16,
    pub payload: Vec<u8>,
}

impl RpcMessage {
    #[allow(unused)]
    pub fn new(net_id: u32, component_index: u8, function_hash: u16, payload: Vec<u8>) -> Self {
        Self {
            net_id,
            component_index,
            function_hash,
            payload,
        }
    }
    #[allow(unused)]
    pub fn get_payload_content(&self) -> Vec<u8> {
        self.payload[4..].to_vec()
    }
}

impl MessageSerializer for RpcMessage {
    fn serialize(&mut self, writer: &mut NetworkWriter)
    where
        Self: Sized,
    {
        writer.write_blittable(Self::get_full_name().hash16());
        writer.write_blittable_compress(self.net_id);
        writer.write_blittable(self.component_index);
        writer.write_blittable(self.function_hash);
        writer.write_slice_and_size(self.payload.as_slice());
    }
}

impl MessageDeserializer for RpcMessage {
    fn deserialize(reader: &mut NetworkReader) -> Self
    where
        Self: Sized,
    {
        let net_id = reader.read_blittable_compress();
        let component_index = reader.read_blittable();
        let function_hash = reader.read_blittable();
        let payload = reader.read_slice_and_size();
        Self {
            net_id,
            component_index,
            function_hash,
            payload: payload.to_vec(),
        }
    }
}
