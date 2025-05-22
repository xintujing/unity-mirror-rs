use crate::mirror::messages::message::{MessageDeserializer, MessageSerializer, OnMessageHandler};
use crate::mirror::namespace::Namespace;
use crate::mirror::network_reader::NetworkReader;
use crate::mirror::network_writer::NetworkWriter;
use crate::mirror::stable_hash::StableHash;
use dda_macro::{namespace, Message};

#[namespace(prefix = "Mirror")]
#[derive(Debug, PartialEq, Clone, Default, Message)]
pub struct ObjectSpawnFinishedMessage;

impl ObjectSpawnFinishedMessage {
    #[allow(unused)]
    pub fn new() -> Self {
        Self
    }
}

impl MessageSerializer for ObjectSpawnFinishedMessage {
    fn serialize(&mut self, writer: &mut NetworkWriter)
    where
        Self: Sized,
    {
        writer.write_blittable(Self::get_full_path().hash16());
    }
}

impl MessageDeserializer for ObjectSpawnFinishedMessage {
    fn deserialize(_: &mut NetworkReader) -> Self
    where
        Self: Sized,
    {
        Self
    }
}

impl OnMessageHandler for ObjectSpawnFinishedMessage {}
