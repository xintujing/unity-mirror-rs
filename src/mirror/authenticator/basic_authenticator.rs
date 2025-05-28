use crate::commons::object::Object;
use crate::commons::revel_arc::RevelArc;
use crate::mirror::authenticator::authenticator::Authenticator;
use crate::mirror::messages::message::{MessageDeserializer, MessageSerializer};
use crate::mirror::network_connection::NetworkConnection;
use crate::mirror::network_reader::NetworkReader;
use crate::mirror::network_writer::NetworkWriter;
use crate::mirror::stable_hash::StableHash;
use crate::mirror::transport::TransportChannel;
use crate::mirror::NetworkServer;
use unity_mirror_macro::{namespace, Message};

#[namespace(prefix = "Mirror.Authenticators.BasicAuthenticator+")]
#[derive(Default, Clone, Message)]
pub struct AuthRequestMessage {
    auth_username: String,
    auth_password: String,
}
impl AuthRequestMessage {
    fn on_auth_request_message(
        conn: &mut RevelArc<NetworkConnection>,
        message: &AuthRequestMessage,
        channel: TransportChannel,
    ) {
        // TODO: Implement authentication logic here
        // match authenticator_factory() {
        //     None => {
        //         conn.send_message(
        //             &mut AuthResponseMessage {
        //                 code: 200,
        //                 message: "Invalid Credentials".to_string(),
        //             },
        //             channel,
        //         );
        //         // 请检查是否正确注册了 Authenticator
        //         log::error!("[AuthRequestMessage.handle] Authenticator not registered");
        //         self.server_reject(conn);
        //     }
        //     Some(_authenticator) => {
        //         conn.is_authenticated = true;
        //         conn.send_message(
        //             &mut AuthResponseMessage {
        //                 code: 100,
        //                 message: "Success".to_string(),
        //             },
        //             channel,
        //         );
        //         self.server_accept(conn);
        //     }
        // }
    }
}

impl Authenticator for AuthRequestMessage {
    fn new() -> Box<dyn Authenticator> {
        Box::new(AuthRequestMessage {
            auth_username: "".to_string(),
            auth_password: "".to_string(),
        })
    }

    fn on_start_server(&self) {
        NetworkServer.register_handler::<AuthRequestMessage>(Self::on_auth_request_message, false);
    }

    fn on_stop_server(&self) {
        NetworkServer.unregister_handler::<AuthRequestMessage>();
    }

    fn on_server_authenticate(&self, _connection: &mut NetworkConnection) {
        // do nothing...wait for AuthRequestMessage from client
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
