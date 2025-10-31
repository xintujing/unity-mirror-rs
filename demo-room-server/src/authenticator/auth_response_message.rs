use unity_mirror_rs::macro_namespace::*;
use unity_mirror_rs::macro_network_message::*;
use unity_mirror_rs::mirror::message::{MessageDeserializer, MessageSerializer};
use unity_mirror_rs::mirror::{NetworkReader, NetworkWriter, StableHash};

#[namespace(prefix = "DaDaA.Authenticator.Authenticator+")]
#[derive(Debug, Default, NetworkMessage)]
pub struct AuthResponseMessage {
    pub code: u8,
    pub message: String,
}

impl MessageSerializer for AuthResponseMessage {
    fn serialize(&mut self, writer: &mut NetworkWriter)
    where
        Self: Sized,
    {
        writer.write_blittable(Self::get_full_name().hash16());
        writer.write_blittable(self.code);
        writer.write_str(&self.message);
    }
}

impl MessageDeserializer for AuthResponseMessage {
    fn deserialize(reader: &mut NetworkReader) -> Self
    where
        Self: Sized,
    {
        Self {
            code: reader.read_blittable(),
            message: reader.read_string(),
        }
    }
}
