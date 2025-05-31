use crate::commons::action::SelfMutAction;
use crate::commons::revel_arc::RevelArc;
use crate::commons::revel_weak::RevelWeak;
use crate::mirror::batching::un_batcher_pool::UnBatcherPool;
use crate::mirror::messages::command_message::CommandMessage;
use crate::mirror::messages::entity_state_message::EntityStateMessage;
use crate::mirror::messages::message::{Message, MessageHandler, MessageHandlerFuncType, ID_SIZE};
use crate::mirror::messages::network_ping_message::NetworkPingMessage;
use crate::mirror::messages::network_pong_message::NetworkPongMessage;
use crate::mirror::messages::object_spawn_finished_message::ObjectSpawnFinishedMessage;
use crate::mirror::messages::object_spawn_started_message::ObjectSpawnStartedMessage;
use crate::mirror::messages::ready_message::ReadyMessage;
use crate::mirror::messages::time_snapshot_message::TimeSnapshotMessage;
use crate::mirror::network_connection::NetworkConnection;
use crate::mirror::network_reader::NetworkReader;
use crate::mirror::network_reader_pool::NetworkReaderPool;
use crate::mirror::remote_calls::RemoteProcedureCalls;
use crate::mirror::snapshot_interpolation::snapshot_interpolation_settings::SnapshotInterpolationSettings;
use crate::mirror::snapshot_interpolation::time_sample::TimeSample;
use crate::mirror::snapshot_interpolation::time_snapshot::TimeSnapshot;
use crate::mirror::stable_hash::StableHash;
use crate::mirror::transport::TransportChannel::Reliable;
use crate::mirror::transport::{CallbackProcessor, TranSport, TransportChannel, TransportError};
use crate::mirror::{NetworkIdentity, RemoteCallType};
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

    pub next_network_id: u32,

    // Identity
    pub spawned: HashMap<u32, RevelWeak<Box<NetworkIdentity>>>,

    // State
    pub active: bool,

    // Handlers
    message_handlers: HashMap<u16, MessageHandler>,

    // Connections
    pub connections: HashMap<u64, RevelArc<NetworkConnection>>,

    // Events
    pub on_connected_event: SelfMutAction<(RevelArc<NetworkConnection>,), ()>,
    pub on_disconnected_event: SelfMutAction<(RevelArc<NetworkConnection>,), ()>,
    pub on_error_event: SelfMutAction<(RevelArc<NetworkConnection>, TransportError, String), ()>,
    pub on_transport_exception_event:
        SelfMutAction<(RevelArc<NetworkConnection>, Box<dyn std::error::Error>), ()>,
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
    message_handlers: Default::default(),
    next_network_id: 1,
    spawned: Default::default(),
    active: false,
    on_connected_event: SelfMutAction::default(),
    on_disconnected_event: SelfMutAction::default(),
    on_error_event: SelfMutAction::default(),
    on_transport_exception_event: SelfMutAction::default(),
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
        if Self::is_connection_allowed(conn_id, address) {
            let connection = NetworkConnection::new(conn_id, address.to_string());
            Self::on_connected(RevelArc::new(connection));
            return;
        }
        TranSport.active().server_disconnect(conn_id);
    }

    fn is_connection_allowed(conn_id: u64, address: &str) -> bool {
        if !Self.listen {
            log::warn!(
                "Server not listening, rejecting connectionId={} with address={}",
                conn_id,
                address
            );
            return false;
        }

        if conn_id == 0 {
            log::error!("Server.HandleConnect: invalid connectionId={} needs to be != 0, because 0 is reserved for local player.", conn_id);
            return false;
        }

        if Self.connections.contains_key(&conn_id) {
            log::error!(
                "Server connectionId={} already in use. Client with address={} will be kicked",
                conn_id,
                address
            );
            return false;
        }

        if Self.connections.len() as i32 >= Self.max_connections {
            log::error!(
                "Server full, client connectionId={} with address={} will be kicked",
                conn_id,
                address
            );
            return false;
        }
        true
    }

    fn on_connected(conn: RevelArc<NetworkConnection>) {
        Self.add_connection(conn.clone());
        Self.on_connected_event.call((conn,));
    }

    fn connection_contains_key(&self, conn_id: &u64) -> bool {
        self.connections.contains_key(conn_id)
    }

    fn add_connection(&mut self, conn: RevelArc<NetworkConnection>) -> bool {
        if self.connection_contains_key(&conn.id) {
            return false;
        }
        self.connections.insert(conn.id, conn);
        true
    }

    fn remove_connection(&mut self, conn_id: u64) -> Option<RevelArc<NetworkConnection>> {
        self.connections.remove(&conn_id)
    }

    fn on_transport_data(conn_id: u64, data: &[u8], channel: TransportChannel) {
        if let Some(conn) = Self.connections.get(&conn_id) {
            let mut conn = conn.clone();
            UnBatcherPool::get_return(move |un_batcher| {
                if !un_batcher.add_batch_with_slice(data) {
                    if Self.exceptions_disconnect {
                        log::error!(
                        "NetworkServer: received message from connectionId:{} was too short (messages should start with message id). Disconnecting.",
                        conn_id
                    );
                        conn.disconnect();
                    } else {
                        log::warn!(
                        "NetworkServer: received message from connectionId:{} was too short (messages should start with message id).",
                        conn_id
                    );
                        return;
                    }
                }

                if Self.is_loading_scene {
                    log::warn!(
                    "NetworkServer: connectionId:{} is loading scene, skipping message processing.",
                    conn_id
                );
                    return;
                }

                while let Some((message, remote_timestamp)) = un_batcher.get_next_message() {
                    NetworkReaderPool::get_with_slice_return(message, |reader| {
                        if reader.remaining() < ID_SIZE {
                            if Self.exceptions_disconnect {
                                log::error!(
                                "NetworkServer: connectionId:{} received message with invalid header, disconnecting.",
                                conn_id
                            );
                                conn.disconnect();
                            } else {
                                log::warn!(
                                "NetworkServer: connectionId:{} received message with invalid header.",
                                conn_id
                            );
                            }
                            return;
                        }

                        conn.remote_time_stamp = remote_timestamp;

                        if !Self.unpack_and_invoke(conn.clone(), reader, channel) {
                            if Self.exceptions_disconnect {
                                log::error!(
                                "NetworkServer: connectionId:{} received message with unknown type, disconnecting.",
                                conn_id
                            );
                                conn.disconnect();
                            } else {
                                log::warn!(
                                "NetworkServer: connectionId:{} received message with unknown type.",
                                conn_id
                            );
                            }
                            return;
                        }
                    });
                }

                if !Self.is_loading_scene && un_batcher.batches_count() > 0 {
                    log::error!("NetworkServer: connectionId:{} has unprocessed batches, skipping message processing.",conn_id);
                }
            });
        } else {
            log::warn!(
                "NetworkServer: connectionId:{} not found when processing data.",
                conn_id
            );
        }
    }

    fn on_transport_error(conn_id: u64, err: TransportError, reason: &str) {
        log::warn!(
            "NetworkServer: connectionId:{} encountered an error: {}. Reason: {}",
            conn_id,
            err,
            reason
        );
        if let Some(conn) = Self.connections.get(&conn_id) {
            Self.on_error_event
                .call((conn.clone(), err, reason.to_string()));
        }
    }

    fn on_transport_exception(conn_id: u64, _err: Box<dyn std::error::Error>) {
        log::warn!(
            "NetworkServer: connectionId:{} encountered a transport exception.",
            conn_id
        );
        if let Some(conn) = Self.connections.get(&conn_id) {
            Self.on_transport_exception_event.call((conn.clone(), _err));
        }
    }

    fn on_transport_disconnected(conn_id: u64) {
        if let Some(mut conn) = Self.remove_connection(conn_id) {
            conn.cleanup();
            if Self.on_disconnected_event.is_registered() {
                Self.on_disconnected_event.call((conn.clone(),));
            } else {
                // TODO DestroyPlayerForConnection(conn)
            }
        }
    }

    fn register_message_handlers(&mut self) {
        self.register_handler::<ReadyMessage>(Self::on_client_ready_message, true);
        self.register_handler::<CommandMessage>(Self::on_client_command_message, true);
        self.register_handler::<NetworkPingMessage>(Self::on_client_network_ping_message, false);
        self.register_handler::<NetworkPongMessage>(Self::on_client_network_pong_message, false);
        self.register_handler::<EntityStateMessage>(Self::on_client_entity_state_message, true);
        self.register_handler::<TimeSnapshotMessage>(Self::on_client_time_snapshot_message, false);
    }

    fn on_client_ready_message(
        connection: RevelArc<NetworkConnection>,
        _: ReadyMessage,
        _: TransportChannel,
    ) {
        Self::set_client_ready(connection);
    }

    pub fn set_client_ready(mut connection: RevelArc<NetworkConnection>) {
        connection.is_ready = true;
        if connection.identity.upgradable() {
            Self::spawn_observers_for_connection(connection);
        }
    }

    fn spawn_observers_for_connection(mut connection: RevelArc<NetworkConnection>) {
        if !connection.is_ready {
            return;
        }

        connection.send_message(&mut ObjectSpawnStartedMessage::default(), Reliable);

        // TODO: Spawn observers logic

        connection.send_message(&mut ObjectSpawnFinishedMessage::default(), Reliable);
    }

    fn on_client_command_message(
        connection: RevelArc<NetworkConnection>,
        message: CommandMessage,
        channel: TransportChannel,
    ) {
        if !connection.is_ready {
            if channel == Reliable {
                if let Some(weak_net_identity) = Self.spawned.get(&message.net_id) {
                    if let Some(net_identity) = weak_net_identity.get() {
                        if message.component_index < net_identity.network_behaviours().len() as u8 {
                            if let Some(name) =
                                RemoteProcedureCalls.get_function_method_name(message.function_hash)
                            {
                                log::warn!(
                                    "Command {} received for {} [netId={}] component index={} when client not ready. This may be ignored if client intentionally set NotReady.",
                                    name,
                                    net_identity.name(),
                                    message.net_id,
                                    message.component_index
                                );
                                return;
                            }

                            log::warn!(
                                "Command received from {} while client is not ready. This may be ignored if client intentionally set NotReady.",
                                connection.id
                            );
                        }
                    }
                }
            }
            return;
        }

        match Self.spawned.get(&message.net_id) {
            None => {
                if channel == Reliable {
                    log::warn!(
                        "Spawned object not found when handling Command message netId={}",
                        message.net_id
                    );
                }

                return;
            }
            Some(weak_net_identity) => {
                if let Some(net_identity) = weak_net_identity.get() {
                    let requires_authority =
                        RemoteProcedureCalls.command_requires_authority(&message.function_hash);
                    let is_owner = connection.ptr_eq_weak(&net_identity.connection());

                    if requires_authority && !is_owner {
                        if (message.component_index as usize)
                            < net_identity.network_behaviours().len()
                        {
                            if let Some(name) =
                                RemoteProcedureCalls.get_function_method_name(message.function_hash)
                            {
                                log::warn!(
                                    "Command {} received for {} [netId={}] component index={} when client not ready. This may be ignored if client intentionally set NotReady.",
                                    name,
                                    net_identity.name(),
                                    message.net_id,
                                    message.component_index
                                );
                                return;
                            }

                            log::warn!(
                                "Command received from {} while client is not ready. This may be ignored if client intentionally set NotReady.",
                                connection.id
                            );
                        }
                        return;
                    }

                    NetworkReaderPool::get_with_slice_return(
                        message.payload.as_slice(),
                        |reader| {
                            net_identity.handle_remote_call(
                                message.component_index,
                                message.function_hash,
                                RemoteCallType::Command,
                                reader,
                                connection,
                            );
                        },
                    );
                }
            }
        }
    }

    fn on_client_network_ping_message(
        mut connection: RevelArc<NetworkConnection>,
        message: NetworkPingMessage,
        _: TransportChannel,
    ) {
        let local_time = Time::unscaled_time_f64();
        let unadjusted_error = local_time - message.local_time;
        let adjusted_error = local_time - message.predicted_time_adjusted;

        let mut pong_message =
            NetworkPongMessage::new(message.local_time, unadjusted_error, adjusted_error);
        connection.send_message(&mut pong_message, Reliable);
    }

    fn on_client_network_pong_message(
        mut connection: RevelArc<NetworkConnection>,
        message: NetworkPongMessage,
        _: TransportChannel,
    ) {
        let local_time = Time::unscaled_time_f64();
        if message.local_time > local_time {
            return;
        }

        let new_rtt = local_time - message.local_time;
        connection._rtt.add(new_rtt);
    }

    fn on_client_entity_state_message(
        mut connection: RevelArc<NetworkConnection>,
        message: EntityStateMessage,
        _: TransportChannel,
    ) {
        match Self.spawned.get(&message.net_id) {
            None => {
                log::warn!(
                    "EntityStateMessage from {} for netId={} without authority.",
                    connection.id,
                    message.net_id
                );
            }
            Some(weak_net_identity) => {
                if let Some(net_identity) = weak_net_identity.get() {
                    if !connection.ptr_eq_weak(&net_identity.connection()) {
                        log::warn!(
                            "EntityStateMessage from {} for {} without authority.",
                            connection.id,
                            net_identity.name()
                        );
                        return;
                    }
                    NetworkReaderPool::get_with_slice_return(
                        message.payload.as_slice(),
                        |reader| {
                            if !net_identity.deserialize_server(reader) {
                                if Self.exceptions_disconnect {
                                    log::error!(
                                        "Server failed to deserialize client state for {} with netId={}. Disconnecting.",
                                        net_identity.name(),
                                        net_identity.net_id()
                                    );
                                    connection.disconnect();
                                } else {
                                    log::warn!(
                                        "Server failed to deserialize client state for {} with netId={}.",
                                        net_identity.name(),
                                        net_identity.net_id()
                                    );
                                }
                            } else {
                                log::warn!(
                                "Server failed to deserialize client state for {} with netId={}.",
                                net_identity.name(),
                                net_identity.net_id());
                            }
                        },
                    );
                }
            }
        }
    }

    fn on_client_time_snapshot_message(
        mut connection: RevelArc<NetworkConnection>,
        _: TimeSnapshotMessage,
        _: TransportChannel,
    ) {
        let remote_time_stamp = connection.remote_time_stamp;
        connection.on_time_snapshot(TimeSnapshot::new(
            remote_time_stamp,
            Time::unscaled_time_f64(),
        ))
    }

    fn unpack_and_invoke(
        &mut self,
        mut connection: RevelArc<NetworkConnection>,
        reader: &mut NetworkReader,
        channel: TransportChannel,
    ) -> bool {
        if let Some(msg_type) = MessageHandler::unpack_id(reader) {
            return match self.message_handlers.get_mut(&msg_type) {
                None => {
                    log::warn!("No handler registered for message type: {}", msg_type);
                    false
                }
                Some(handler) => {
                    connection.last_message_time = Time::unscaled_time_f64();
                    handler.invoke(connection, reader, channel);
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
        if self.message_handlers.contains_key(&message_id) {
            log::warn!(
                "Handler for message {} already registered, please use replace_handler instead.",
                M::get_full_name()
            );
            return;
        }
        self.message_handlers.insert(
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
        self.message_handlers.insert(
            message_id,
            MessageHandler::new(func, require_authentication),
        );
    }

    pub fn unregister_handler<M>(&mut self)
    where
        M: Message + 'static,
    {
        let message_id = M::get_full_name().hash16();
        self.message_handlers.remove(&message_id);
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

        self.connections.clear();
        self.message_handlers.clear();
        self.cleanup_spawned();
        self.active = false;
        NetworkIdentity::reset_server_statics();

        self.on_connected_event.reset();
        self.on_disconnected_event.reset();
        self.on_error_event.reset();
        self.on_transport_exception_event.reset();
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
