use crate::macro_namespace::*;
use crate::macro_network_message::*;
use crate::mirror::messages::message::{MessageDeserializer, MessageSerializer};
use crate::mirror::stable_hash::StableHash;
use crate::mirror::NetworkReader;
use crate::mirror::NetworkWriter;

#[namespace(prefix = "Mirror")]
#[derive(Debug, PartialEq, Clone, Default, NetworkMessage)]
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
