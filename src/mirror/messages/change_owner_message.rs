use crate::commons::Object;
use crate::mirror::messages::message::{MessageDeserializer, MessageSerializer};
use crate::mirror::NetworkReader;
use crate::mirror::NetworkWriter;
use crate::mirror::stable_hash::StableHash;
use crate::{namespace, NetworkMessage};

#[namespace(prefix = "Mirror")]
#[derive(Debug, PartialEq, Clone, Default, NetworkMessage)]
pub struct ChangeOwnerMessage {
    pub net_id: u32,
    pub is_owner: bool,
    pub is_local_player: bool,
}

impl ChangeOwnerMessage {
    #[allow(unused)]
    pub(crate) fn new(net_id: u32, is_owner: bool, is_local_player: bool) -> Self {
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
        writer.write_blittable_compress(self.net_id);
        writer.write_blittable(self.is_owner);
        writer.write_blittable(self.is_local_player);
    }
}

impl MessageDeserializer for ChangeOwnerMessage {
    fn deserialize(reader: &mut NetworkReader) -> Self
    where
        Self: Sized,
    {
        let net_id = reader.read_blittable_compress();
        let is_owner = reader.read_blittable();
        let is_local_player = reader.read_blittable();
        Self {
            net_id,
            is_owner,
            is_local_player,
        }
    }
}
