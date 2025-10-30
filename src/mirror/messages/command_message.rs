use crate::macro_namespace::*;
use crate::macro_network_message::*;
use crate::mirror::messages::message::{MessageDeserializer, MessageSerializer};
use crate::mirror::stable_hash::StableHash;
use crate::mirror::NetworkReader;
use crate::mirror::NetworkWriter;

#[namespace(prefix = "Mirror")]
#[derive(Debug, PartialEq, Clone, Default, NetworkMessage)]
pub struct CommandMessage {
    pub net_id: u32,
    pub component_index: u8,
    pub function_hash: u16,
    pub payload: Vec<u8>,
}

impl CommandMessage {
    #[allow(unused)]
    pub(crate) fn new(
        net_id: u32,
        component_index: u8,
        function_hash: u16,
        payload: Vec<u8>,
    ) -> Self {
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