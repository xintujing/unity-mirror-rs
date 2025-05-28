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

impl MessageSerializer for NetworkPingMessage {
    fn serialize(&mut self, writer: &mut NetworkWriter)
    where
        Self: Sized,
    {
        writer.write_blittable(Self::get_full_name().hash16());
        writer.write_blittable(self.local_time);
        writer.write_blittable(self.predicted_time_adjusted);
    }
}

impl MessageDeserializer for NetworkPingMessage {
    fn deserialize(reader: &mut NetworkReader) -> Self
    where
        Self: Sized,
    {
        let local_time = reader.read_blittable();
        let predicted_time_adjusted = reader.read_blittable();
        Self {
            local_time,
            predicted_time_adjusted,
        }
    }
}

impl OnMessageHandler for NetworkPingMessage {
    fn handle(&self, conn: &mut NetworkConnection, channel: TransportChannel) {}
}
