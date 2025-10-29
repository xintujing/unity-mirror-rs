use unity_mirror_rs::macro_namespace::*;
use unity_mirror_rs::macro_network_message::*;
use unity_mirror_rs::mirror::message::{MessageDeserializer, MessageSerializer};
use unity_mirror_rs::mirror::{NetworkReader, NetworkWriter, StableHash};

#[namespace(prefix = "DaDaA.Authenticator.Authenticator+")]
#[derive(Debug, Default, Clone, NetworkMessage)]
pub struct AuthRequestMessage {
    pub auth_username: String,
    pub auth_password: String,
    pub input_name: String,
    pub input_points: i32,
    pub player_prefab: String,
}

impl MessageSerializer for AuthRequestMessage {
    fn serialize(&mut self, writer: &mut NetworkWriter)
    where
        Self: Sized,
    {
        writer.write_blittable(Self::get_full_name().hash16());
        writer.write_str(&self.auth_username);
        writer.write_str(&self.auth_password);
        writer.write_str(&self.input_name);
        writer.write_blittable_compress(self.input_points);
        writer.write_str(&self.player_prefab);
    }
}

impl MessageDeserializer for AuthRequestMessage {
    fn deserialize(reader: &mut NetworkReader) -> Self
    where
        Self: Sized,
    {
        Self {
            auth_username: reader.read_string(),
            auth_password: reader.read_string(),
            input_name: reader.read_string(),
            input_points: reader.read_blittable_compress(),
            player_prefab: reader.read_string(),
        }
    }
}
