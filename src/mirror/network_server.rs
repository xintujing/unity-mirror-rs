use crate::commons::revel_arc::RevelArc;
use crate::commons::revel_weak::RevelWeak;
use crate::mirror::messages::command_message::CommandMessage;
use crate::mirror::messages::entity_state_message::EntityStateMessage;
use crate::mirror::messages::message::{Message, MessageHandler, MessageHandlerFuncType};
use crate::mirror::messages::network_ping_message::NetworkPingMessage;
use crate::mirror::messages::network_pong_message::NetworkPongMessage;
use crate::mirror::messages::ready_message::ReadyMessage;
use crate::mirror::messages::time_snapshot_message::TimeSnapshotMessage;
use crate::mirror::network_connection::NetworkConnection;
use crate::mirror::network_reader::NetworkReader;
use crate::mirror::snapshot_interpolation::snapshot_interpolation_settings::SnapshotInterpolationSettings;
use crate::mirror::snapshot_interpolation::time_sample::TimeSample;
use crate::mirror::stable_hash::StableHash;
use crate::mirror::transport::{CallbackProcessor, TranSport, TransportChannel, TransportError};
use crate::mirror::NetworkIdentity;
use crate::unity_engine::Time;
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
    handlers: HashMap<u16, MessageHandler>,
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
    handlers: Default::default(),
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
        self.register_message_handlers();
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

    fn register_message_handlers(&mut self) {
        // TODO: 注册消息处理器
        self.register_handler::<ReadyMessage>(Self::on_client_ready_message, true);
        self.register_handler::<CommandMessage>(Self::on_client_command_message, true);
        self.register_handler::<NetworkPingMessage>(Self::on_client_network_ping_message, false);
        self.register_handler::<NetworkPongMessage>(Self::on_client_network_pong_message, false);
        self.register_handler::<EntityStateMessage>(Self::on_client_entity_state_message, true);
        self.register_handler::<TimeSnapshotMessage>(Self::on_client_time_snapshot_message, false);
    }

    fn on_client_ready_message(
        connection: &mut RevelArc<NetworkConnection>,
        _: &ReadyMessage,
        _: TransportChannel,
    ) {
        // TODO: 处理客户端准备就绪消息
    }

    fn on_client_command_message(
        connection: &mut RevelArc<NetworkConnection>,
        _: &CommandMessage,
        _: TransportChannel,
    ) {
        // TODO: 处理客户端命令消息
    }

    fn on_client_network_ping_message(
        connection: &mut RevelArc<NetworkConnection>,
        _: &NetworkPingMessage,
        _: TransportChannel,
    ) {
        // TODO: 处理客户端网络Ping消息
    }

    fn on_client_network_pong_message(
        connection: &mut RevelArc<NetworkConnection>,
        _: &NetworkPongMessage,
        _: TransportChannel,
    ) {
        // TODO: 处理客户端网络Pong消息
    }

    fn on_client_entity_state_message(
        connection: &mut RevelArc<NetworkConnection>,
        _: &EntityStateMessage,
        _: TransportChannel,
    ) {
        // TODO: 处理客户端实体状态消息
    }

    fn on_client_time_snapshot_message(
        connection: &mut RevelArc<NetworkConnection>,
        _: &TimeSnapshotMessage,
        _: TransportChannel,
    ) {
        // TODO: 处理时间快照消息
    }

    fn unpack_and_invoke(
        &mut self,
        connection: &mut RevelArc<NetworkConnection>,
        reader: &mut NetworkReader,
        channel: TransportChannel,
    ) -> bool {
        if let Some(msg_type) = MessageHandler::unpack_id(reader) {
            return match self.handlers.get_mut(&msg_type) {
                None => {
                    log::warn!("No handler registered for message type: {}", msg_type);
                    false
                }
                Some(handler) => {
                    handler.invoke(connection, reader, channel);
                    connection.last_message_time = Time::unscaled_time_f64();
                    true
                }
            };
        }
        log::warn!("Invalid message header for connection:{}", connection.id);
        false
    }

    pub fn register_handler<M>(
        &mut self,
        func: MessageHandlerFuncType<M>,
        require_authentication: bool,
    ) where
        M: Message + 'static,
    {
        let message_id = M::get_full_name().hash16();
        if self.handlers.contains_key(&message_id) {
            log::warn!(
                "Handler for message {} already registered, please use replace_handler instead.",
                M::get_full_name()
            );
            return;
        }
        self.handlers.insert(
            message_id,
            MessageHandler::new::<M>(func, require_authentication),
        );
    }

    pub fn replace_handler<M>(
        &mut self,
        func: MessageHandlerFuncType<M>,
        require_authentication: bool,
    ) where
        M: Message + 'static,
    {
        let message_id = M::get_full_name().hash16();
        self.handlers.insert(
            message_id,
            MessageHandler::new(func, require_authentication),
        );
    }

    pub fn unregister_handler<M>(&mut self)
    where
        M: Message + 'static,
    {
        let message_id = M::get_full_name().hash16();
        self.handlers.remove(&message_id);
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
