use crate::macro_namespace::*;
use crate::macro_network_message::*;
use crate::mirror::messages::message::{MessageDeserializer, MessageSerializer};
use crate::mirror::stable_hash::StableHash;
use crate::mirror::NetworkReader;
use crate::mirror::NetworkWriter;

#[namespace(prefix = "Mirror")]
#[derive(Debug, PartialEq, Clone, Default, NetworkMessage)]
pub struct ObjectHideMessage {
    pub net_id: u32,
}

impl ObjectHideMessage {
    #[allow(unused)]
    pub fn new(net_id: u32) -> Self {
        Self { net_id }
    }
}