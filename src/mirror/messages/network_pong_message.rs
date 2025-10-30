use crate::macro_namespace::*;
use crate::macro_network_message::*;
use crate::mirror::messages::message::{MessageDeserializer, MessageSerializer};
use crate::mirror::stable_hash::StableHash;
use crate::mirror::NetworkReader;
use crate::mirror::NetworkWriter;

#[namespace(prefix = "Mirror")]
#[derive(Debug, PartialEq, Clone, Default, Copy, NetworkMessage)]
pub struct NetworkPongMessage {
    pub local_time: f64,
    pub prediction_error_unadjusted: f64,
    pub prediction_error_adjusted: f64,
}

impl NetworkPongMessage {
    #[allow(unused)]
    pub fn new(
        local_time: f64,
        prediction_error_unadjusted: f64,
        prediction_error_adjusted: f64,
    ) -> NetworkPongMessage {
        Self {
            local_time,
            prediction_error_unadjusted,
            prediction_error_adjusted,
        }
    }
}
