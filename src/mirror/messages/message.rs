use crate::commons::object::Object;
use crate::mirror::connect::Connection;
use crate::mirror::network_reader::NetworkReader;
use crate::mirror::network_writer::NetworkWriter;
use crate::mirror::stable_hash::StableHash;
use crate::mirror::transport::TransportChannel;
use once_cell::sync::Lazy;
use std::collections::HashMap;

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

#[allow(unused)]
pub trait OnMessageHandler {
    fn handle(&self, conn: &mut Connection, channel: TransportChannel) {}
}

pub trait Message:
    Object + Default + MessageSerializer + MessageDeserializer + OnMessageHandler
{
}

static mut ON_MESSAGE_HANDLER_REGISTERS: Lazy<
    HashMap<u16, fn(&mut Connection, &mut NetworkReader, TransportChannel)>,
> = Lazy::new(|| HashMap::new());

pub fn register_messages<M>()
where
    M: Message + Sync + Send + 'static,
{
    let hash = M::get_full_name().hash16();
    let full_name = M::get_full_name();
    #[allow(static_mut_refs)]
    unsafe {
        if ON_MESSAGE_HANDLER_REGISTERS.contains_key(&hash) {
            panic!("Message '{full_name}' already registered")
        }
        ON_MESSAGE_HANDLER_REGISTERS.insert(hash, |connection, reader, channel| {
            match M::get_full_name().hash16() {
                // TimeSnapshotMessage
                57097 => {}
                // NetworkPingMessage
                17487 => {}
                // NetworkPongMessage
                27095 => {}
                _ => {
                    // println!("received message: {}", M::full_name());
                }
            }
            let mut m = M::deserialize(reader);
            M::handle(&mut m, connection, channel)
        });
    }
}

pub fn unpack_message(
    conn: &mut Connection,
    reader: &mut NetworkReader,
    channel: TransportChannel,
) -> bool {
    // 读取消息 ID
    let message_id = reader.read_blittable::<u16>();

    #[allow(static_mut_refs)]
    if let Some(f) = unsafe { ON_MESSAGE_HANDLER_REGISTERS.get(&message_id) } {
        f(conn, reader, channel);
        return true;
    }
    log::error!(
        "ON_MESSAGE_HANDLER_REGISTERS not found `message_id: {:02X}",
        message_id
    );
    false
}

pub const ID_SIZE: usize = size_of::<u16>();

pub fn max_message_size(channel: TransportChannel) -> usize {
    max_content_size(channel) + ID_SIZE
}

pub fn max_content_size(channel: TransportChannel) -> usize {
    1500
}
