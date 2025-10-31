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