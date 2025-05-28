use crate::commons::revel_arc::RevelArc;
use crate::commons::revel_weak::RevelWeak;
use crate::mirror::network_connection::NetworkConnection;
use crate::mirror::snapshot_interpolation::snapshot_interpolation_settings::SnapshotInterpolationSettings;
use crate::mirror::snapshot_interpolation::time_sample::TimeSample;
use crate::mirror::transport::{CallbackProcessor, TranSport, TransportChannel, TransportError};
use crate::mirror::NetworkIdentity;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

#[allow(unused)]
pub struct NetworkServerStatic {
    initialized: bool,
    address: &'static str,
    port: u16,
    listen: bool,
    pub max_connections: i32,
    // 发送速率
    pub tick_rate: u32,
    // 完整更新持续时间
    full_update_duration: TimeSample,
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

    pub client_snapshot_settings: SnapshotInterpolationSettings,

    pub connections: HashMap<u64, RevelArc<NetworkConnection>>,
    pub next_network_id: u32,
    pub spawned: HashMap<u32, RevelWeak<Box<NetworkIdentity>>>,
    pub active: bool,
}

static mut CONFIG: Lazy<NetworkServerStatic> = Lazy::new(|| NetworkServerStatic {
    initialized: false,
    address: "0.0.0.0",
    port: 7777,
    listen: true,
    max_connections: 0,
    tick_rate: 30,
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
    client_snapshot_settings: SnapshotInterpolationSettings::new(),
    connections: Default::default(),
    next_network_id: 1,
    spawned: Default::default(),
    active: false,
});

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

pub struct NetworkServer;

impl NetworkServer {
    pub fn send_rate(&self) -> i32 {
        self.tick_rate as i32
    }

    pub fn tick_interval(&self) -> f64 {
        match self.tick_rate < i32::MAX as u32 {
            true => 1.0 / self.tick_rate as f64,
            false => 0.0,
        }
    }

    pub fn send_interval(&self) -> f64 {
        match self.tick_rate < i32::MAX as u32 {
            true => 1.0 / self.tick_rate as f64,
            false => 0.0,
        }
    }
}

impl NetworkServer {
    pub fn listen(&mut self, max_connections: i32) {
        if self.initialized {
            log::warn!("NetworkServer is already initialized.");
            return;
        }
        self.connections.clear();
        self.add_transport_handlers();
        self.initialized = true;

        self.early_update_duration = TimeSample::new(self.send_rate() as u32);
        self.late_update_duration = TimeSample::new(self.send_rate() as u32);
        self.full_update_duration = TimeSample::new(self.send_rate() as u32);
        self.max_connections = max_connections;
        if self.listen {
            TranSport.active().server_start((self.address, self.port));
        }
        self.active = true;
    }

    fn add_transport_handlers(&self) {
        let processor = CallbackProcessor {
            on_server_connected: Self::on_transport_connected,
            on_server_connected_with_address: Self::on_transport_connected_with_address,
            on_server_data_received: Self::on_transport_data,
            on_server_data_sent: |_, _, _| {},
            on_server_error: Self::on_transport_error,
            on_server_transport_exception: Self::on_transport_exception,
            on_server_disconnected: Self::on_transport_disconnected,
        };
        TranSport.active().init(processor);
    }

    fn remove_transport_handlers(&self) {
        let processor = CallbackProcessor {
            on_server_connected: |_| {},
            on_server_connected_with_address: |_, _| {},
            on_server_data_received: |_, _, _| {},
            on_server_data_sent: |_, _, _| {},
            on_server_error: |_, _, _| {},
            on_server_transport_exception: |_, _| {},
            on_server_disconnected: |_| {},
        };
        TranSport.active().init(processor);
    }

    fn on_transport_connected(conn_id: u64) {
        Self::on_transport_connected_with_address(
            conn_id,
            TranSport
                .active()
                .server_get_client_address(conn_id)
                .unwrap_or_default()
                .as_str(),
        );
    }

    fn on_transport_connected_with_address(conn_id: u64, address: &str) {
        // TODO 处理连接
    }

    fn on_transport_data(conn_id: u64, data: &[u8], channel: TransportChannel) {
        // TODO: 处理接收到的数据
    }

    fn on_transport_error(conn_id: u64, err: TransportError, reason: &str) {
        // TODO: 处理传输错误
    }

    fn on_transport_exception(conn_id: u64, _err: Box<dyn std::error::Error>) {
        // TODO: 处理传输异常
    }

    fn on_transport_disconnected(conn_id: u64) {
        // TODO: 处理断开连接
    }

    pub fn shutdown(&mut self) {
        if self.initialized {
            self.disconnect_all();
            TranSport.active().server_stop();

            self.remove_transport_handlers();

            self.initialized = false;
        }
        self.listen = true;
        self.is_loading_scene = false;
        self.late_send_time = 0.0;
        self.actual_tick_rate = 0;

        // TODO handlers.Clear();
        self.connections.clear();
        self.cleanup_spawned();
        self.active = false;
        NetworkIdentity::reset_server_statics();

        // TODO Event
        // OnConnectedEvent = null;
        // OnDisconnectedEvent = null;
        // OnErrorEvent = null;
        // OnTransportExceptionEvent = null;
    }

    fn disconnect_all(&mut self) {
        for conn in self.connections.values_mut() {
            conn.disconnect();
        }
        self.connections.clear();
    }

    fn cleanup_spawned(&mut self) {
        // todo: 清理已生成的对象
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

impl Deref for NetworkServer {
    type Target = NetworkServerStatic;

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
