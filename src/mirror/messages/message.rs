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

// #[allow(unused)]
// pub trait OnMessageHandler {
//     fn handle(&self, conn: &mut RevelArc<NetworkConnection>, channel: TransportChannel) {}
// }

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

// static mut ON_MESSAGE_HANDLER_REGISTERS: Lazy<
//     HashMap<u16, fn(&mut RevelArc<NetworkConnection>, &mut NetworkReader, TransportChannel)>,
// > = Lazy::new(|| HashMap::new());
//
// pub fn register_messages<M>()
// where
//     M: Message + Sync + Send + 'static,
// {
//     let hash = M::get_full_name().hash16();
//     let full_name = M::get_full_name();
//     #[allow(static_mut_refs)]
//     unsafe {
//         if ON_MESSAGE_HANDLER_REGISTERS.contains_key(&hash) {
//             panic!("Message '{full_name}' already registered")
//         }
//         ON_MESSAGE_HANDLER_REGISTERS.insert(hash, |connection, reader, channel| {
//             match M::get_full_name().hash16() {
//                 // TimeSnapshotMessage
//                 57097 => {}
//                 // NetworkPingMessage
//                 17487 => {}
//                 // NetworkPongMessage
//                 27095 => {}
//                 _ => {
//                     // println!("received message: {}", M::full_name());
//                 }
//             }
//             let mut m = M::deserialize(reader);
//             M::handle(&mut m, connection, channel)
//         });
//     }
// }

// pub fn unpack_message(
//     conn: &mut RevelArc<NetworkConnection>,
//     reader: &mut NetworkReader,
//     channel: TransportChannel,
// ) -> bool {
//     // 读取消息 ID
//     let message_id = reader.read_blittable::<u16>();
//
//     #[allow(static_mut_refs)]
//     if let Some(f) = unsafe { ON_MESSAGE_HANDLER_REGISTERS.get(&message_id) } {
//         f(conn, reader, channel);
//         return true;
//     }
//     log::error!(
//         "ON_MESSAGE_HANDLER_REGISTERS not found `message_id: {:02X}",
//         message_id
//     );
//     false
// }

pub type MessageHandlerFuncType<M: Message> =
    fn(&mut RevelArc<NetworkConnection>, &M, TransportChannel);
type MessageHandlerWrappedFuncType =
    Box<dyn FnMut(&mut RevelArc<NetworkConnection>, &dyn Message, TransportChannel)>;

pub struct MessageHandler {
    wrapped_func: MessageHandlerWrappedFuncType,
    require_authentication: bool,
}

impl MessageHandler {
    pub fn new<M: Message + 'static>(
        func: MessageHandlerFuncType<M>,
        require_authentication: bool,
    ) -> Self {
        // 将泛型函数包装为动态分发函数
        let wrapped_func: MessageHandlerWrappedFuncType =
            Box::new(move |conn, dyn_msg, channel| {
                // 使用 `downcast_ref` 将 `dyn Message` 转换回具体类型
                if let Some(msg) = dyn_msg.as_any().downcast_ref::<M>() {
                    func(conn, msg, channel)
                } else {
                    panic!("Message type mismatch!");
                }
            });
        Self {
            wrapped_func,
            require_authentication,
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
