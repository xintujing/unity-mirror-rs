use crate::commons::action::SelfMutAction;
use crate::commons::RevelArc;
use crate::macro_authenticator_factory::*;
use crate::macro_namespace::*;
use crate::macro_network_message::*;
use crate::mirror::messages::message::{MessageDeserializer, MessageSerializer};
use crate::mirror::stable_hash::StableHash;
use crate::mirror::transport::TransportChannel;
use crate::mirror::AuthenticatorBase;
use crate::mirror::NetworkReader;
use crate::mirror::NetworkWriter;
use crate::mirror::{Authenticator, NetworkConnectionToClient, NetworkServer};
use crate::unity_engine::MonoBehaviour;

#[namespace(prefix = "Mirror.Authenticators")]
#[authenticator_factory]
#[derive(Default)]
pub struct BasicAuthenticator {}

impl BasicAuthenticator {
    pub fn on_auth_request_message(&mut self, mut connection: RevelArc<Box<NetworkConnectionToClient>>, _message: BasicAuthenticatorRequestMessage, channel: TransportChannel) {
        {
            // TODO: Implement your authentication logic here
            let auth_response_message = BasicAuthenticatorResponseMessage {
                code: 100,
                message: "Success".to_string(),
            };
            connection.send_message(auth_response_message, channel);
        }
        self.server_accept(connection);
    }
}

impl MonoBehaviour for BasicAuthenticator {}

impl Authenticator for BasicAuthenticator {
    fn new() -> Self {
        let authenticator = Self::default();
        authenticator
    }

    fn on_start_server(&self) {
        NetworkServer.register_handler::<BasicAuthenticatorRequestMessage>(SelfMutAction::new(self.weak.clone(), Self::on_auth_request_message), false);
    }

    fn on_stop_server(&self) {
        NetworkServer.unregister_handler::<BasicAuthenticatorRequestMessage>();
    }

    fn on_server_authenticate(&self, _connection: RevelArc<Box<NetworkConnectionToClient>>) {
        // do nothing...wait for BasicAuthenticatorRequestMessage from client
    }
}

// BasicAuthenticator AuthRequestMessage
#[namespace(
    prefix = "Mirror.Authenticators.BasicAuthenticator+",
    rename = "AuthRequestMessage"
)]
#[derive(Default, Clone, NetworkMessage)]
pub struct BasicAuthenticatorRequestMessage {
    auth_username: String,
    auth_password: String,
}

impl MessageSerializer for BasicAuthenticatorRequestMessage {
    fn serialize(&mut self, writer: &mut NetworkWriter)
    where
        Self: Sized,
    {
        writer.write_blittable(Self::get_full_name().hash16());
        writer.write_str(&self.auth_username);
        writer.write_str(&self.auth_password);
    }
}

impl MessageDeserializer for BasicAuthenticatorRequestMessage {
    fn deserialize(reader: &mut NetworkReader) -> Self
    where
        Self: Sized,
    {
        Self {
            auth_username: reader.read_string(),
            auth_password: reader.read_string(),
        }
    }
}

// BasicAuthenticator AuthResponseMessage
#[namespace(
    prefix = "Mirror.Authenticators.BasicAuthenticator+",
    rename = "AuthResponseMessage"
)]
#[derive(Default, Clone, NetworkMessage)]
pub struct BasicAuthenticatorResponseMessage {
    code: u8,
    message: String,
}

impl MessageSerializer for BasicAuthenticatorResponseMessage {
    fn serialize(&mut self, writer: &mut NetworkWriter)
    where
        Self: Sized,
    {
        writer.write_blittable(Self::get_full_name().hash16());
        writer.write_blittable(self.code);
        writer.write_str(self.message.as_str());
    }
}

impl MessageDeserializer for BasicAuthenticatorResponseMessage {
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
