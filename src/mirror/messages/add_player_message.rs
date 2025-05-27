use crate::commons::object::Object;
use crate::mirror::connect::Connection;
use crate::mirror::messages::message::{MessageDeserializer, MessageSerializer, OnMessageHandler};
use crate::mirror::network_reader::NetworkReader;
use crate::mirror::network_writer::NetworkWriter;
use crate::mirror::stable_hash::StableHash;
use crate::mirror::transport::TransportChannel;
use unity_mirror_macro::{namespace, MessageRegistry};

#[namespace(prefix = "Mirror")]
#[derive(Debug, PartialEq, Clone, Default, MessageRegistry)]
pub struct AddPlayerMessage;

impl AddPlayerMessage {
    #[allow(unused)]
    fn new() -> Self {
        Self
    }
}

impl MessageSerializer for AddPlayerMessage {
    fn serialize(&mut self, writer: &mut NetworkWriter)
    where
        Self: Sized,
    {
        writer.write_blittable(Self::get_full_name().hash16());
    }
}

impl MessageDeserializer for AddPlayerMessage {
    fn deserialize(reader: &mut NetworkReader) -> Self
    where
        Self: Sized,
    {
        let _ = reader;
        Self
    }
}

impl OnMessageHandler for AddPlayerMessage {
    fn handle(&self, conn: &mut Connection, _: TransportChannel) {
        // NetworkManager::on_server_add_player_internal(uc_conn, self);
    }
}
