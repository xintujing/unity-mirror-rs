use crate::mirror::authenticator::authenticator::Authenticator;
use crate::mirror::authenticator::basic_authenticator::AuthResponseMessage;
use crate::mirror::batching::batcher::Batcher;
use crate::mirror::transport::TransportChannel;
use std::collections::HashMap;

pub struct Connection {
    pub id: u64,
    pub is_authenticated: bool,
    pub authentication_data: Option<Box<dyn Authenticator>>,
    pub is_ready: bool,
    pub last_message_time: f32,

    // Batcher,
    batches: HashMap<TransportChannel, Batcher>,
}

impl Connection {
    pub(crate) fn send_message(
        &self,
        auth_response_message: &mut AuthResponseMessage,
        transport_channel: TransportChannel,
    ) {
        todo!()
    }
}

impl Connection {
    pub(crate) fn disconnect(&self) {
        todo!()
    }
}
