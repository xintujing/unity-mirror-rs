use crate::commons::action::SelfMutAction;
use crate::commons::object::Object;
use crate::commons::revel_arc::RevelArc;
use crate::commons::revel_weak::RevelWeak;
use crate::mirror::messages::message::{MessageDeserializer, MessageSerializer};
use crate::mirror::network_connection::NetworkConnection;
use crate::mirror::network_reader::NetworkReader;
use crate::mirror::network_writer::NetworkWriter;
use crate::mirror::stable_hash::StableHash;
use crate::mirror::transport::TransportChannel;
use crate::mirror::{Authenticator, NetworkServer};
use crate::unity_engine::{MonoBehaviour, MonoBehaviourAny};
use std::any::Any;
use unity_mirror_macro::{namespace, AuthenticatorFactory, Message};

#[namespace(prefix = "Mirror.Authenticators")]
#[derive(AuthenticatorFactory, Default)]
pub struct BasicAuthenticator {
    weak_self: RevelWeak<Box<Self>>,
    on_server_authenticated: SelfMutAction<(RevelArc<NetworkConnection>,), ()>,
}
impl BasicAuthenticator {
    pub fn on_auth_request_message(
        &mut self,
        mut connection: RevelArc<NetworkConnection>,
        message: BasicAuthenticatorRequestMessage,
        channel: TransportChannel,
    ) {
    }
}

impl MonoBehaviour for BasicAuthenticator {}

impl Authenticator for BasicAuthenticator {
    fn new() -> Box<dyn Authenticator> {
        Box::new(BasicAuthenticator {
            weak_self: Default::default(),
            on_server_authenticated: Default::default(),
        })
    }

    fn on_start_server(&self) {
        NetworkServer.register_handler::<BasicAuthenticatorRequestMessage>(
            SelfMutAction::new(self.weak_self.clone(), Self::on_auth_request_message),
            false,
        );
    }

    fn on_stop_server(&self) {
        NetworkServer.unregister_handler::<BasicAuthenticatorRequestMessage>();
    }

    fn set_on_server_authenticated(
        &mut self,
        event: SelfMutAction<(RevelArc<NetworkConnection>,), ()>,
    ) {
        self.on_server_authenticated = event;
    }

    fn get_on_server_authenticated(&self) -> &SelfMutAction<(RevelArc<NetworkConnection>,), ()> {
        &self.on_server_authenticated
    }

    fn on_server_authenticate(&self, connection: RevelArc<NetworkConnection>) {
        // do nothing...wait for BasicAuthenticatorRequestMessage from client
    }

    fn set_weak_self(&mut self, weak_self: RevelWeak<Box<dyn Authenticator>>) {
        if let Some(weak_self) = weak_self.downcast::<Self>() {
            self.weak_self = weak_self.clone();
        }
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
