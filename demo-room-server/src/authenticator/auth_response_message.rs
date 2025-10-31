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


