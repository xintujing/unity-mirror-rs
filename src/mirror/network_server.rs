use crate::commons::revel_arc::RevelArc;
use crate::commons::revel_weak::RevelWeak;
use crate::mirror::network_connection::NetworkConnection;
use crate::mirror::snapshot_interpolation::time_sample::TimeSample;
use crate::mirror::transport::{init_transport_manager, TransportChannel, TransportError};
use crate::transports::kcp2k2_transport::Kcp2kTransport;
use kcp2k_rust::kcp2k_config::Kcp2KConfig;
use lazy_static::lazy_static;
use once_cell::sync::Lazy;
use std::cell::{RefCell, UnsafeCell};
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::AtomicBool;

lazy_static! {
    static ref ACTIVE: AtomicBool = AtomicBool::new(false);
}
static mut CONFIG: Lazy<NetworkServerConfig> = Lazy::new(|| NetworkServerConfig {
    tick_rate: 30,
    send_rate: 30,
    full_update_duration: TimeSample::new(30),
    late_send_time: 0.0,
    disconnect_inactive_connections: false,
    disconnect_inactive_timeout: 60.0,
    actual_tick_rate_counter: 0,
    actual_tick_rate_start: 0.0,
    actual_tick_rate: 0,
    early_update_duration: TimeSample::new(0),
    late_update_duration: TimeSample::new(0),
    is_loading_scene: false,
    exceptions_disconnect: true,
});

static mut CONNECTIONS: Lazy<HashMap<String, RevelArc<NetworkConnection>>> =
    Lazy::new(|| HashMap::default());

#[allow(unused)]
pub enum ReplacePlayerOptions {
    KeepAuthority,
    KeepActive,
    UnSpawn,
    Destroy,
}

#[allow(unused)]
pub enum RemovePlayerOptions {
    /// <summary> player对象在服务器和客户端上保持活动状态。仅删除所有权</summary>
    KeepActive,
    /// <summary> player对象在客户端上取消使用，但仍在服务器上</summary>
    UnSpawn,
    /// <summary>播放器对象在服务器和客户端上被破坏</summary>
    Destroy,
}

pub struct NetworkServerConfig {
    // 发送速率
    tick_rate: u32,
    send_rate: i32,
    // 完整更新持续时间
    full_update_duration: TimeSample,
    // 发送间隔
    // send_interval: f32,
    late_send_time: f64,

    disconnect_inactive_connections: bool,
    disconnect_inactive_timeout: f32,

    actual_tick_rate_counter: i32,
    actual_tick_rate_start: f64,
    actual_tick_rate: i32,

    early_update_duration: TimeSample,
    late_update_duration: TimeSample,

    is_loading_scene: bool,
    exceptions_disconnect: bool,
}

pub struct NetworkServer;

impl Deref for NetworkServer {
    type Target = NetworkServerConfig;

    fn deref(&self) -> &Self::Target {
        #[allow(static_mut_refs)]
        unsafe {
            &*CONFIG
        }
    }
}
impl DerefMut for NetworkServer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        #[allow(static_mut_refs)]
        unsafe {
            &mut *CONFIG
        }
    }
}

// impl NetworkServer {
//     pub fn start(&mut self) {
//         init_transport_manager(
//             Kcp2kTransport::new(Some(Kcp2KConfig {
//                 ..Kcp2KConfig::default()
//             })),
//             Self::on_server_connected,
//             Self::on_server_connected_with_address,
//             Self::on_server_data_received,
//             Self::on_server_data_sent,
//             Self::on_server_error,
//             Self::on_server_transport_exception,
//             Self::on_server_disconnected,
//         );
//     }
//
//     fn on_server_connected(conn_id: u64) {
//         if NetworkServer::connection_contains_key(&conn_id) {
//             return;
//         }
//
//         // 新建连接
//         let arc_coon = new_arc_mutex(Connection::new(conn_id));
//
//         Self::on_connected_event(&arc_coon);
//
//         NetworkServer::add_connection(arc_coon);
//     }
//     fn on_server_connected_with_address(conn_id: u64, addr: &str) {
//         println!("on_server_connected_with_address: {} {}", conn_id, addr);
//     }
//
//     fn on_server_data_received(connection_id: u64, data: &[u8], channel: TransportChannel) {
//         if !NetworkServer::connection_contains_key(&connection_id) {
//             log::error!("HandleData Unknown connectionId: {}", connection_id);
//             return;
//         }
//
//         UnBatcherPool::get_return(|un_batcher| {
//             NetworkServer::get_connection(&connection_id, |connection| {
//                 if !un_batcher.add_batch_with_array_segment(data) {
//                     println!("on_server_data_received: un_batcher.add_batch_with_bytes failed");
//                     connection.disconnect();
//                     return;
//                 }
//             });
//
//             while let Some((message_data, timestamp)) = un_batcher.get_next_message() {
//                 NetworkReaderPool::get_with_array_segment_return(message_data, |reader| {
//                     if reader.remaining() >= size_of::<u16>() {
//                         NetworkServer::get_connection(&connection_id, |connection| {
//                             connection.set_remote_time_stamp(timestamp);
//                         });
//
//                         NetworkServer::get_arc_connection(&connection_id, |connection| {
//                             if !message::unpack_message(connection, reader, channel.into()) {
//                                 match NetworkServer::config().exceptions_disconnect {
//                                     true => {
//                                         NetworkServer::get_connection(&connection_id, |conn| {
//                                             log::error!(
//                                             "NetworkServer: failed to unpack and invoke message. Disconnecting {}.",
//                                             connection_id
//                                         );
//                                             conn.disconnect();
//                                         });
//                                     }
//                                     false => {
//                                         log::error!(
//                                         "NetworkServer: failed to unpack and invoke message from connectionId:{}.",
//                                         connection_id
//                                     );
//                                     }
//                                 }
//                             };
//                         });
//                     }
//                 });
//             }
//         });
//     }
//     fn on_server_data_sent(_conn_id: u64, _data: &[u8], _channel: TransportChannel) {}
//
//     fn on_server_error(conn_id: u64, err: TransportError, reason: &str) {
//         println!("on_server_error: {} {} {}", conn_id, err, reason);
//         if let Some(conn) = NetworkServer::get_arc_mutex_connection(&conn_id) {
//             Self::on_error_event(&conn, err, reason);
//         }
//     }
//     fn on_server_transport_exception(conn_id: u64, err: Box<dyn std::error::Error>) {
//         println!("on_server_transport_exception: {} {}", conn_id, err);
//         if let Some(conn) = NetworkServer::get_arc_mutex_connection(&conn_id) {
//             Self::on_transport_exception_event(&conn, err);
//         }
//     }
//
//     // OnServerDisconnected
//     fn on_server_disconnected(conn_id: u64) {
//         unsafe {
//             if let Some(connection) = CONNECTIONS.remove(&conn_id.to_string()) {
//                 connection.cleanup();
//             }
//         }
//
//         if let Some(conn) = NetworkServer::get_arc_mutex_connection(&conn_id) {
//             if let Ok(mut connection) = conn.lock() {
//                 connection.cleanup()
//             }
//
//             Self::remove_connection(conn_id);
//
//             Self::on_disconnected_event(&conn);
//             qwe!();
//             println!("on_server_disconnected: {}", conn_id);
//         }
//     }
// }
//
// lazy_static::lazy_static! {
//     static ref VIRTUAL_NETWORK_SERVER_ON_CONNECTED_EVENT: RevelArc<
//         fn(connection: RevelArc<NetworkConnection>),
//     > = RevelArc::new(NetworkServer::super_on_connected_event);
//
//     static ref VIRTUAL_NETWORK_SERVER_ON_DISCONNECTED_EVENT: RevelArc<
//         fn(connection: RevelArc<NetworkConnection>),
//     > = RevelArc::new(NetworkServer::super_on_disconnected_event);
//
//     static ref VIRTUAL_NETWORK_SERVER_ON_ERROR_EVENT: RevelArc<
//         fn(_connection: RevelArc<NetworkConnection>, _error: TransportError, _reason: &str),
//     > = RevelArc::new(NetworkServer::super_on_error_event);
//
//     static ref VIRTUAL_NETWORK_SERVER_ON_TRANSPORT_EXCEPTION_EVENT: RevelArc<
//         fn(_connection: RevelArc<NetworkConnection>, _exception: Box<dyn std::error::Error>),
//     > = RevelArc::new(NetworkServer::super_on_transport_exception_event);
// }
//
// impl NetworkServer {
//     // #[virtual_func]
//     pub(crate) fn super_on_connected_event(connection: RevelArc<NetworkConnection>) {
//         if let Ok(mut connection) = connection.lock() {
//             let message = NotReadyMessage::new();
//             connection.send_message(message, TransportChannel::Reliable);
//         }
//     }
//     // #[virtual_func]
//     pub(crate) fn super_on_disconnected_event(connection: RevelArc<NetworkConnection>) {
//         Self::destroy_player_for_connection(connection);
//     }
//     // #[virtual_func]
//     pub(crate) fn super_on_error_event(
//         _connection: RevelArc<NetworkConnection>,
//         _error: TransportError,
//         _reason: &str,
//     ) {
//     }
//     // #[virtual_func]
//     pub(crate) fn super_on_transport_exception_event(
//         _connection: RevelArc<NetworkConnection>,
//         _exception: Box<dyn std::error::Error>,
//     ) {
//     }
//     pub(crate) fn on_connected_event(connection: RevelArc<NetworkConnection>) {
//         VIRTUAL_NETWORK_SERVER_ON_CONNECTED_EVENT.to_owned()(connection)
//     }
//     pub(crate) fn replace_on_connected_event(f: fn(connection: RevelArc<NetworkConnection>)) {
//         *VIRTUAL_NETWORK_SERVER_ON_CONNECTED_EVENT.get() = f;
//     }
//     pub(crate) fn on_disconnected_event(connection: RevelArc<NetworkConnection>) {
//         VIRTUAL_NETWORK_SERVER_ON_DISCONNECTED_EVENT.to_owned()(connection)
//     }
//     pub(crate) fn replace_on_disconnected_event(f: fn(connection: RevelArc<NetworkConnection>)) {
//         *VIRTUAL_NETWORK_SERVER_ON_DISCONNECTED_EVENT.get() = f;
//     }
//     pub(crate) fn on_error_event(
//         _connection: RevelArc<NetworkConnection>,
//         _error: TransportError,
//         _reason: &str,
//     ) {
//         VIRTUAL_NETWORK_SERVER_ON_ERROR_EVENT.to_owned()(_connection, _error, _reason)
//     }
//     pub(crate) fn replace_on_error_event(
//         f: fn(_connection: RevelArc<NetworkConnection>, _error: TransportError, _reason: &str),
//     ) {
//         *VIRTUAL_NETWORK_SERVER_ON_ERROR_EVENT.get() = f;
//     }
//     pub(crate) fn on_transport_exception_event(
//         _connection: RevelArc<NetworkConnection>,
//         _exception: Box<dyn std::error::Error>,
//     ) {
//         VIRTUAL_NETWORK_SERVER_ON_TRANSPORT_EXCEPTION_EVENT.to_owned()(_connection, _exception)
//     }
//     pub(crate) fn replace_on_transport_exception_event(
//         f: fn(_connection: RevelArc<NetworkConnection>, _exception: Box<dyn std::error::Error>),
//     ) {
//         *VIRTUAL_NETWORK_SERVER_ON_TRANSPORT_EXCEPTION_EVENT.get() = f;
//     }
// }
