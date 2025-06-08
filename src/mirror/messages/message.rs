use crate::commons::action::SelfMutAction;
use crate::commons::object::Object;
use crate::commons::revel_arc::RevelArc;
use crate::mirror::NetworkConnection;
use crate::mirror::NetworkReader;
use crate::mirror::NetworkWriter;
use crate::mirror::NetworkConnectionToClient;
use crate::mirror::transport::TransportChannel;

pub trait MessageSerializer {
    fn serialize(&mut self, writer: &mut NetworkWriter)
    where
        Self: Sized;
}
pub trait MessageDeserializer {
    fn deserialize(#[allow(unused)] reader: &mut NetworkReader) -> Self
    where
        Self: Sized;
}

pub trait NetworkMessage: Object + MessageSerializer + MessageDeserializer {}

// pub type MessageHandlerFuncType<M> = fn(RevelArc<NetworkConnection>, M, TransportChannel);

type MessageHandlerWrappedFuncType =
Box<dyn FnMut(RevelArc<Box<NetworkConnectionToClient>>, &mut NetworkReader, TransportChannel)>;

pub struct MessageHandler {
    #[allow(unused)]
    wrapped_func: MessageHandlerWrappedFuncType,
    #[allow(unused)]
    pub require_authentication: bool,
}

impl MessageHandler {
    pub fn new<M: NetworkMessage + 'static>(
        func: SelfMutAction<(RevelArc<Box<NetworkConnectionToClient>>, M, TransportChannel), ()>,
        require_authentication: bool,
    ) -> Self {
        // 将泛型函数包装为动态分发函数
        let wrapped_func: MessageHandlerWrappedFuncType = Box::new(move |conn, reader, channel| {
            let msg = M::deserialize(reader);
            func.call((conn, msg, channel))
        });
        Self {
            wrapped_func,
            require_authentication,
        }
    }

    #[allow(unused)]
    pub fn invoke(
        &mut self,
        mut conn: RevelArc<Box<NetworkConnectionToClient>>,
        reader: &mut NetworkReader,
        channel: TransportChannel,
    ) {
        if self.require_authentication && !conn.is_authenticated {
            log::warn!("Disconnecting connection: {}. Received message that required authentication, but the user has not authenticated yet",conn.connection_id);
            conn.disconnect.call(());
            return;
        }
        (self.wrapped_func)(conn, reader, channel);
    }

    #[allow(unused)]
    pub fn unpack_id(reader: &mut NetworkReader) -> Option<u16> {
        let msg_type = reader.read_blittable::<u16>();
        match msg_type {
            0 => None,
            _ => Some(msg_type),
        }
    }
}

pub const ID_SIZE: usize = size_of::<u16>();

pub fn max_message_size(channel: TransportChannel) -> usize {
    max_content_size(channel) + ID_SIZE
}

pub fn max_content_size(_channel: TransportChannel) -> usize {
    1500
}
