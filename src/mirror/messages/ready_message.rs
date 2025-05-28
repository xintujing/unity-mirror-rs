use crate::commons::object::Object;
use crate::mirror::network_connection::NetworkConnection;
use crate::mirror::messages::message::{MessageDeserializer, MessageSerializer, OnMessageHandler};
use crate::mirror::network_reader::NetworkReader;
use crate::mirror::network_writer::NetworkWriter;
use crate::mirror::stable_hash::StableHash;
use crate::mirror::transport::TransportChannel;
use unity_mirror_macro::{namespace, MessageRegistry};
use crate::commons::revel_arc::RevelArc;

#[namespace(prefix = "Mirror")]
#[derive(Debug, PartialEq, Clone, Default, MessageRegistry)]
pub struct ReadyMessage;

impl MessageSerializer for ReadyMessage {
    fn serialize(&mut self, writer: &mut NetworkWriter)
    where
        Self: Sized,
    {
        writer.write_blittable(Self::get_full_name().hash16());
    }
}

impl MessageDeserializer for ReadyMessage {
    fn deserialize(_: &mut NetworkReader) -> Self
    where
        Self: Sized,
    {
        Self
    }
}

impl OnMessageHandler for ReadyMessage {
    fn handle(&self, conn: &mut RevelArc<NetworkConnection>, _channel: TransportChannel) {
        // NetworkManager::on_server_ready_message_internal(connection, self.clone());

        // log::info!(
        //     "AddPlayerMessage received from connection {} on channel {:?}",
        //     connection_id,
        //     channel
        // )
    }
}
