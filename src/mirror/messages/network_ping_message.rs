use crate::macro_namespace::*;
use crate::macro_network_message::*;
use crate::mirror::messages::message::{MessageDeserializer, MessageSerializer};
use crate::mirror::stable_hash::StableHash;
use crate::mirror::NetworkReader;
use crate::mirror::NetworkWriter;

#[namespace(prefix = "Mirror")]
#[derive(Debug, PartialEq, Clone, Default, NetworkMessage)]
pub struct NetworkPingMessage {
    pub local_time: f64,
    pub predicted_time_adjusted: f64,
}

impl NetworkPingMessage {
    #[allow(unused)]
    pub fn new(local_time: f64, predicted_time_adjusted: f64) -> Self {
        Self {
            local_time,
            predicted_time_adjusted,
        }
    }
}
