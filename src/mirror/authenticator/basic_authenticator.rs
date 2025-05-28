use crate::commons::object::Object;
use crate::mirror::authenticator::authenticator::{authenticator_factory, Authenticator};
use crate::mirror::connect::Connection;
use crate::mirror::messages::message::{MessageDeserializer, MessageSerializer, OnMessageHandler};
use crate::mirror::network_reader::NetworkReader;
use crate::mirror::network_writer::NetworkWriter;
use crate::mirror::stable_hash::StableHash;
use crate::mirror::transport::TransportChannel;
use unity_mirror_macro::{namespace, Message, MessageRegistry};

#[namespace(prefix = "Mirror.Authenticators.BasicAuthenticator+")]
#[derive(Default, Clone, MessageRegistry)]
pub struct AuthRequestMessage {
    auth_username: String,
    auth_password: String,
}

impl Authenticator for AuthRequestMessage {
    fn new() -> Box<dyn Authenticator> {
        Box::new(AuthRequestMessage {
            auth_username: "".to_string(),
            auth_password: "".to_string(),
        })
    }

    fn on_start_server(&self) {}

    fn on_stop_server(&self) {}

    fn on_server_authenticate(&self, _connection: &mut Connection) {
        // do nothing...wait for AuthRequestMessage from client
    }
}

impl OnMessageHandler for AuthRequestMessage {
    fn handle(&self, conn: &mut Connection, channel: TransportChannel) {
        match authenticator_factory() {
            None => {
                conn.send_message(
                    &mut AuthResponseMessage {
                        code: 200,
                        message: "Invalid Credentials".to_string(),
                    },
                    channel,
                );
                // 请检查是否正确注册了 Authenticator
                log::error!("[AuthRequestMessage.handle] Authenticator not registered");
                self.server_reject(conn);
            }
            Some(_authenticator) => {
                conn.is_authenticated = true;
                conn.send_message(
                    &mut AuthResponseMessage {
                        code: 100,
                        message: "Success".to_string(),
                    },
                    channel,
                );
                self.server_accept(conn);
            }
        }
    }
}

impl MessageSerializer for AuthRequestMessage {
    fn serialize(&mut self, writer: &mut NetworkWriter)
    where
        Self: Sized,
    {
        writer.write_blittable(Self::get_full_name().hash16());
        writer.write_str(&self.auth_username);
        writer.write_str(&self.auth_password);
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
        }
    }
}

#[namespace(prefix = "Mirror.Authenticators.BasicAuthenticator+")]
#[derive(Default, Clone, Message)]
pub struct AuthResponseMessage {
    code: u8,
    message: String,
}

impl MessageSerializer for AuthResponseMessage {
    fn serialize(&mut self, writer: &mut NetworkWriter)
    where
        Self: Sized,
    {
        writer.write_blittable(Self::get_full_name().hash16());
        writer.write_blittable(self.code);
        writer.write_str(self.message.as_str());
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

impl OnMessageHandler for AuthResponseMessage {}
