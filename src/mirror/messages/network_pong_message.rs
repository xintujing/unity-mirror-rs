use crate::commons::object::Object;
use crate::mirror::connect::Connection;
use crate::mirror::messages::message::{MessageDeserializer, MessageSerializer, OnMessageHandler};
use crate::mirror::network_reader::NetworkReader;
use crate::mirror::network_writer::NetworkWriter;
use crate::mirror::stable_hash::StableHash;
use crate::mirror::transport::TransportChannel;
use unity_mirror_macro::{namespace, MessageRegistry};

#[namespace(prefix = "Mirror")]
#[derive(Debug, PartialEq, Clone, Default, Copy, MessageRegistry)]
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

impl MessageSerializer for NetworkPongMessage {
    fn serialize(&mut self, writer: &mut NetworkWriter)
    where
        Self: Sized,
    {
        writer.write_blittable(Self::get_full_name().hash16());
        writer.write_blittable(self.local_time);
        writer.write_blittable(self.prediction_error_unadjusted);
        writer.write_blittable(self.prediction_error_adjusted);
    }
}

impl MessageDeserializer for NetworkPongMessage {
    fn deserialize(reader: &mut NetworkReader) -> Self
    where
        Self: Sized,
    {
        let local_time = reader.read_blittable();
        let prediction_error_unadjusted = reader.read_blittable();
        let prediction_error_adjusted = reader.read_blittable();
        Self {
            local_time,
            prediction_error_unadjusted,
            prediction_error_adjusted,
        }
    }
}

impl OnMessageHandler for NetworkPongMessage {
    fn handle(&self, conn: &mut Connection, channel: TransportChannel) {}
}
