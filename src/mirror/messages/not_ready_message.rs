use crate::commons::object::Object;
use crate::mirror::messages::message::{MessageDeserializer, MessageSerializer, OnMessageHandler};
use crate::mirror::network_reader::NetworkReader;
use crate::mirror::network_writer::NetworkWriter;
use crate::mirror::stable_hash::StableHash;
use unity_mirror_macro::{namespace, Message};

#[namespace(prefix = "Mirror")]
#[derive(Debug, PartialEq, Clone, Default, Message)]
pub struct NotReadyMessage;

impl NotReadyMessage {
    #[allow(unused)]
    pub(crate) fn new() -> Self {
        Self
    }
}

impl MessageSerializer for NotReadyMessage {
    fn serialize(&mut self, writer: &mut NetworkWriter)
    where
        Self: Sized,
    {
        writer.write_blittable(Self::get_full_name().hash16());
    }
}

impl MessageDeserializer for NotReadyMessage {
    fn deserialize(_: &mut NetworkReader) -> Self
    where
        Self: Sized,
    {
        Self
    }
}

impl OnMessageHandler for NotReadyMessage {}
