use unity_mirror_macro::namespace;
use crate::mirror::messages::message::{MessageDeserializer, MessageSerializer, OnMessageHandler};
use crate::mirror::network_reader::NetworkReader;
use crate::mirror::network_writer::NetworkWriter;
use crate::mirror::transport::TransportChannel;

#[namespace(prefix = "Mirror")]
#[derive(Debug, PartialEq, Clone, Default, MessageRegistry)]
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
        writer.write_blittable(Self::get_full_path().hash16());
        writer.write_var_uint(self.net_id);
        writer.write_array_segment_and_size(self.payload.as_slice());
    }
}

impl MessageDeserializer for EntityStateMessage {
    fn deserialize(reader: &mut NetworkReader) -> Self
    where
        Self: Sized,
    {
        let net_id = reader.read_var_uint();
        let payload = reader.read_bytes_and_size();
        Self { net_id, payload }
    }
}

#[allow(unused)]
impl OnMessageHandler for EntityStateMessage {
    fn handle(&self, connection: &ArcUc<Connection>, channel: TransportChannel) {
        println!("EntityStateMessage::handle");
        NetworkServer::on_entity_state_message(self, connection)
    }
}
