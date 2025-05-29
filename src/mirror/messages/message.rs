use crate::commons::as_any::MyAsAny;
use crate::commons::object::Object;
use crate::commons::revel_arc::RevelArc;
use crate::mirror::network_connection::NetworkConnection;
use crate::mirror::network_reader::NetworkReader;
use crate::mirror::network_writer::NetworkWriter;
use crate::mirror::transport::TransportChannel;
use std::any::Any;

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

pub trait Message: Object + MyAsAny + MessageSerializer + MessageDeserializer {}

impl<T: Message + 'static> MyAsAny for T {
    fn as_any(&self) -> &dyn Any
    where
        Self: Sized,
    {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any
    where
        Self: Sized,
    {
        self
    }
}

pub type MessageHandlerFuncType<M> = fn(&mut RevelArc<NetworkConnection>, M, TransportChannel);
type MessageHandlerWrappedFuncType =
    Box<dyn FnMut(&mut RevelArc<NetworkConnection>, &mut NetworkReader, TransportChannel)>;

pub struct MessageHandler {
    #[allow(unused)]
    wrapped_func: MessageHandlerWrappedFuncType,
    #[allow(unused)]
    pub require_authentication: bool,
}

impl MessageHandler {
    pub fn new<M: Message + 'static>(
        func: MessageHandlerFuncType<M>,
        require_authentication: bool,
    ) -> Self {
        // 将泛型函数包装为动态分发函数
        let wrapped_func: MessageHandlerWrappedFuncType = Box::new(move |conn, reader, channel| {
            let msg = M::deserialize(reader);
            func(conn, msg, channel)
        });
        Self {
            wrapped_func,
            require_authentication,
        }
    }

    #[allow(unused)]
    pub fn invoke(
        &mut self,
        conn: &mut RevelArc<NetworkConnection>,
        reader: &mut NetworkReader,
        channel: TransportChannel,
    ) {
        if self.require_authentication && !conn.is_authenticated {
            log::warn!("Disconnecting connection: {}. Received message that required authentication, but the user has not authenticated yet",conn.id);
            conn.disconnect();
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
