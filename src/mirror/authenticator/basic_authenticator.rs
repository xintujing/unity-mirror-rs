use crate::commons::object::Object;
use crate::commons::revel_arc::RevelArc;
use crate::mirror::messages::message::{MessageDeserializer, MessageSerializer};
use crate::mirror::network_connection::NetworkConnection;
use crate::mirror::network_reader::NetworkReader;
use crate::mirror::network_writer::NetworkWriter;
use crate::mirror::stable_hash::StableHash;
use crate::mirror::transport::TransportChannel;
use crate::mirror::{Authenticator, NetworkServer};
use unity_mirror_macro::{namespace, AuthenticatorFactory, Message};

#[namespace(prefix = "Mirror.Authenticators")]
#[derive(AuthenticatorFactory)]
pub struct BasicAuthenticator {}
impl BasicAuthenticator {
    pub fn on_auth_request_message(
        mut connection: RevelArc<NetworkConnection>,
        message: BasicAuthenticatorRequestMessage,
        channel: TransportChannel,
    ) {
        connection.id = 555;
    }
}

impl Authenticator for BasicAuthenticator {
    fn new() -> Box<dyn Authenticator> {
        Box::new(BasicAuthenticator {})
    }

    fn on_start_server(&self) {
        NetworkServer.register_handler::<BasicAuthenticatorRequestMessage>(
            Self::on_auth_request_message,
            false,
        );
    }

    fn on_stop_server(&self) {
        NetworkServer.unregister_handler::<BasicAuthenticatorRequestMessage>();
    }

    fn on_server_authenticate(&self, _connection: &mut NetworkConnection) {
        // do nothing...wait for BasicAuthenticatorRequestMessage from client
    }
}

#[namespace(
    prefix = "Mirror.Authenticators.BasicAuthenticator+",
    rename = "AuthRequestMessage"
)]
#[derive(Default, Clone, Message)]
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

#[namespace(
    prefix = "Mirror.Authenticators.BasicAuthenticator+",
    rename = "AuthResponseMessage"
)]
#[derive(Default, Clone, Message)]
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
