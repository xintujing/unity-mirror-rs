use unity_mirror_macro::namespace;
use crate::commons::object::Object;
use crate::mirror::messages::message::{
    MessageDeserializer, MessageSerializer, OnMessageHandler,
};
use crate::mirror::network_reader::NetworkReader;
use crate::mirror::network_writer::NetworkWriter;

#[namespace(prefix = "Mirror")]
#[derive(Debug, PartialEq, Clone, Default)]
pub struct ChangeOwnerMessage {
    pub net_id: u32,
    pub is_owner: bool,
    pub is_local_player: bool,
}

impl ChangeOwnerMessage {
    #[allow(unused)]
    fn new(net_id: u32, is_owner: bool, is_local_player: bool) -> Self {
        Self {
            net_id,
            is_owner,
            is_local_player,
        }
    }
}

impl MessageSerializer for ChangeOwnerMessage {
    fn serialize(&mut self, writer: &mut NetworkWriter)
    where
        Self: Sized,
    {
        writer.write_blittable(Self::get_full_name().hash16());
        writer.write_var_uint(self.net_id);
        writer.write_blittable(self.is_owner);
        writer.write_blittable(self.is_local_player);
    }
}

impl MessageDeserializer for ChangeOwnerMessage {
    fn deserialize(reader: &mut NetworkReader) -> Self
    where
        Self: Sized,
    {
        let net_id = reader.read_var_uint();
        let is_owner = reader.read_blittable();
        let is_local_player = reader.read_blittable();
        Self {
            net_id,
            is_owner,
            is_local_player,
        }
    }
}

impl OnMessageHandler for ChangeOwnerMessage {}
