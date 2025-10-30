use crate::macro_namespace::*;
use crate::macro_network_message::*;
use crate::mirror::messages::message::{MessageDeserializer, MessageSerializer};
use crate::mirror::stable_hash::StableHash;
use crate::mirror::NetworkReader;
use crate::mirror::NetworkWriter;

#[namespace(prefix = "Mirror")]
#[derive(Debug, PartialEq, Clone, Copy, Default, NetworkMessage)]
pub struct TimeSnapshotMessage;

impl TimeSnapshotMessage {
    #[allow(unused)]
    pub fn new() -> Self {
        Self
    }
}