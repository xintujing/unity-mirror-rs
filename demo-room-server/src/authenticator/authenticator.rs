use crate::authenticator::auth_request_message::AuthRequestMessage;
use crate::authenticator::auth_response_message::AuthResponseMessage;
use unity_mirror_rs::commons::action::SelfMutAction;
use unity_mirror_rs::macro_authenticator_factory::*;
use unity_mirror_rs::macro_namespace::*;
use unity_mirror_rs::mirror::AuthenticatorBase;
use unity_mirror_rs::mirror::{Authenticator, NetworkServer, TransportChannel};
use unity_mirror_rs::unity_engine::MonoBehaviour;

#[namespace(prefix = "DaDaA.Authenticator", rename = "Authenticator")]
#[authenticator_factory]
#[derive(Default)]
pub struct LobbySysAuthenticator {}

impl LobbySysAuthenticator {
    pub fn on_auth_request_message(&mut self, mut connection: RevelArc<Box<NetworkConnectionToClient>>, _message: AuthRequestMessage, channel: TransportChannel) {
        {
            let auth_response_message = AuthResponseMessage {
                code: 100,
                message: "Success".to_string(),
            };
            connection.send_message(auth_response_message, channel);
        }
        self.server_accept(connection);
    }
}

impl MonoBehaviour for LobbySysAuthenticator {}

impl Authenticator for LobbySysAuthenticator {
    fn new() -> Self {
        let authenticator = Self::default();
        authenticator
    }

    fn on_start_server(&self) {
        NetworkServer.register_handler::<AuthRequestMessage>(SelfMutAction::new(self.weak.clone(), Self::on_auth_request_message), false);
    }

    fn on_stop_server(&self) {
        NetworkServer.unregister_handler::<AuthResponseMessage>();
    }

    fn on_server_authenticate(&self, _connection: RevelArc<Box<NetworkConnectionToClient>>) {
        // do nothing...wait for BasicAuthenticatorRequestMessage from client
    }
}