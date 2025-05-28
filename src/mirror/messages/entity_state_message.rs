use crate::commons::object::Object;
use crate::mirror::network_connection::NetworkConnection;
use crate::mirror::messages::message::{MessageDeserializer, MessageSerializer, OnMessageHandler};
use crate::mirror::network_reader::NetworkReader;
use crate::mirror::network_writer::NetworkWriter;
use crate::mirror::stable_hash::StableHash;
use crate::mirror::transport::TransportChannel;
use unity_mirror_macro::{namespace, MessageRegistry};

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

#[allow(unused)]
impl OnMessageHandler for EntityStateMessage {
    fn handle(&self, uc_conn: &mut NetworkConnection, channel: TransportChannel) {
        // println!("EntityStateMessage::handle");
        // NetworkServer::on_entity_state_message(self, connection)
    }
}
