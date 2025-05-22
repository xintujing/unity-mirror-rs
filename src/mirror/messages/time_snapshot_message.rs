use crate::mirror::messages::message::{MessageDeserializer, MessageSerializer};
use crate::mirror::namespace::Namespace;
use crate::mirror::network_reader::NetworkReader;
use crate::mirror::network_writer::NetworkWriter;
use crate::mirror::stable_hash::StableHash;
use dda_macro::{namespace, MessageRegistry};

#[namespace(prefix = "Mirror")]
#[derive(Debug, PartialEq, Clone, Copy, Default, MessageRegistry)]
pub struct TimeSnapshotMessage;

impl TimeSnapshotMessage {
    #[allow(unused)]
    pub fn new() -> Self {
        Self
    }
}

impl MessageSerializer for TimeSnapshotMessage {
    fn serialize(&mut self, writer: &mut NetworkWriter)
    where
        Self: Sized,
    {
        writer.write_blittable(Self::get_full_path().hash16());
    }
}

impl MessageDeserializer for TimeSnapshotMessage {
    fn deserialize(_: &mut NetworkReader) -> Self
    where
        Self: Sized,
    {
        Self
    }
}
