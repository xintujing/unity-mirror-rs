use crate::commons::action::SelfMutAction;
use crate::commons::revel_arc::RevelArc;
use crate::commons::revel_weak::RevelWeak;
use crate::mirror::accurate_interval::AccurateInterval;
use crate::mirror::batching::un_batcher_pool::UnBatcherPool;
use crate::mirror::messages::change_owner_message::ChangeOwnerMessage;
use crate::mirror::messages::command_message::CommandMessage;
use crate::mirror::messages::entity_state_message::EntityStateMessage;
use crate::mirror::messages::message::{max_message_size, MessageHandler, NetworkMessage, ID_SIZE};
use crate::mirror::messages::network_ping_message::NetworkPingMessage;
use crate::mirror::messages::network_pong_message::NetworkPongMessage;
use crate::mirror::messages::object_destroy_message::ObjectDestroyMessage;
use crate::mirror::messages::object_hide_message::ObjectHideMessage;
use crate::mirror::messages::object_spawn_finished_message::ObjectSpawnFinishedMessage;
use crate::mirror::messages::object_spawn_started_message::ObjectSpawnStartedMessage;
use crate::mirror::messages::ready_message::ReadyMessage;
use crate::mirror::messages::scene_message::SceneMessage;
use crate::mirror::messages::time_snapshot_message::TimeSnapshotMessage;
use crate::mirror::not_ready_message::NotReadyMessage;
use crate::mirror::remote_calls::RemoteProcedureCalls;
use crate::mirror::snapshot_interpolation::snapshot_interpolation_settings::SnapshotInterpolationSettings;
use crate::mirror::snapshot_interpolation::time_sample::TimeSample;
use crate::mirror::snapshot_interpolation::time_snapshot::TimeSnapshot;
use crate::mirror::spawn_message::SpawnMessage;
use crate::mirror::stable_hash::StableHash;
use crate::mirror::transport::{
    CallbackProcessor, TransportChannel, TransportError, TransportManager,
};
use crate::mirror::NetworkReader;
use crate::mirror::NetworkReaderPool;
use crate::mirror::NetworkTime;
use crate::mirror::NetworkWriter;
use crate::mirror::NetworkWriterPool;
use crate::mirror::{NetworkConnection, Visibility};
use crate::mirror::{NetworkConnectionToClient, NetworkIdentity, RemoteCallType};
use crate::unity_engine::{GameObject, MonoBehaviour, Time, WorldManager};
use lazy_static::lazy_static;
use once_cell::sync::Lazy;
use std::any::Any;
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

    pub(crate) disconnect_inactive_connections: bool,
    pub(crate) disconnect_inactive_timeout: f32,

    actual_tick_rate_counter: i32,
    actual_tick_rate_start: f64,
    actual_tick_rate: i32,

    early_update_duration: TimeSample,
    late_update_duration: TimeSample,

    pub(crate) is_loading_scene: bool,
    pub(crate) exceptions_disconnect: bool,

    pub client_snapshot_settings: SnapshotInterpolationSettings,

    pub next_network_id: u32,

    // Identity
    pub spawned: HashMap<u32, RevelWeak<Box<NetworkIdentity>>>,

    // State
    pub active: bool,

    // Handlers
    message_handlers: HashMap<u16, MessageHandler>,

    // Connections
    pub connections: HashMap<u64, RevelArc<Box<NetworkConnectionToClient>>>,

    // Events
    pub on_connected_event: SelfMutAction<(RevelArc<Box<NetworkConnectionToClient>>,), ()>,
    pub on_disconnected_event: SelfMutAction<(RevelArc<Box<NetworkConnectionToClient>>,), ()>,
    pub on_error_event: SelfMutAction<
        (
            RevelArc<Box<NetworkConnectionToClient>>,
            TransportError,
            String,
        ),
        (),
    >,
    pub on_transport_exception_event: SelfMutAction<
        (
            RevelArc<Box<NetworkConnectionToClient>>,
            Box<dyn std::error::Error>,
        ),
        (),
    >,
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

static mut NETWORK_SERVER: Lazy<RevelArc<Box<NetworkServer>>> =
    Lazy::new(|| RevelArc::new(Box::new(NetworkServer)));
static mut NETWORK_TIME: Lazy<RevelArc<Box<NetworkTime>>> =
    Lazy::new(|| RevelArc::new(Box::new(NetworkTime)));

impl NetworkServer {
    pub fn listen(&mut self, max_connections: i32) {
        self.initialize();

        self.max_connections = max_connections;

        if self.listen {
            TransportManager
                .active
                .server_start((self.address, self.port));
        }
        self.active = true;

        self.register_message_handlers();
    }

    fn initialize(&mut self) {
        if self.initialized {
            return;
        }

        self.connections.clear();

        NetworkTime.reset_statics();

        self.add_transport_handlers();
        self.initialized = true;

        self.early_update_duration = TimeSample::new(self.send_rate() as u32);
        self.late_update_duration = TimeSample::new(self.send_rate() as u32);
        self.full_update_duration = TimeSample::new(self.send_rate() as u32);
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
        TransportManager.active.init(processor);
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
        TransportManager.active.init(processor);
    }

    fn on_transport_connected(conn_id: u64) {
        Self::on_transport_connected_with_address(
            conn_id,
            TransportManager
                .active
                .server_get_client_address(conn_id)
                .unwrap_or_default()
                .as_str(),
        );
    }

    fn on_transport_connected_with_address(conn_id: u64, address: &str) {
        if Self::is_connection_allowed(conn_id, address) {
            let arc_connection = NetworkConnectionToClient::new(conn_id, address.to_string());
            Self::on_connected(arc_connection);
            return;
        }
        TransportManager.active.server_disconnect(conn_id);
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

    fn on_connected(conn: RevelArc<Box<NetworkConnectionToClient>>) {
        Self.add_connection(conn.clone());
        Self.on_connected_event.call((conn,));
    }

    fn connection_contains_key(&self, conn_id: &u64) -> bool {
        self.connections.contains_key(conn_id)
    }

    fn add_connection(&mut self, conn: RevelArc<Box<NetworkConnectionToClient>>) -> bool {
        if self.connection_contains_key(&conn.connection_id) {
            return false;
        }
        self.connections.insert(conn.connection_id, conn);
        true
    }

    fn remove_connection(
        &mut self,
        conn_id: u64,
    ) -> Option<RevelArc<Box<NetworkConnectionToClient>>> {
        self.connections.remove(&conn_id)
    }

    fn on_transport_data(conn_id: u64, data: &[u8], channel: TransportChannel) {
        if let Some(conn) = Self.connections.get(&conn_id) {
            let mut conn = conn.clone();
            UnBatcherPool::get_by_closure(move |un_batcher| {
                if !un_batcher.add_batch_with_slice(data) {
                    if Self.exceptions_disconnect {
                        log::error!(
                        "NetworkServer: received message from connectionId:{} was too short (messages should start with message id). Disconnecting.",
                        conn_id
                    );
                        conn.disconnect.call(());
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
                                conn.disconnect.call(());
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
                                conn.disconnect.call(());
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
                Self::destroy_player_for_connection(conn)
            }
        }
    }

    pub fn destroy_player_for_connection(mut conn: RevelArc<Box<NetworkConnectionToClient>>) {
        conn.destroy_owned_objects();

        conn.remove_from_observings_observers();

        conn.identity = RevelWeak::default();
    }

    fn register_message_handlers(&mut self) {
        #[allow(static_mut_refs)]
        let weak_network_server = unsafe { NETWORK_SERVER.downgrade() };
        self.register_handler::<ReadyMessage>(
            SelfMutAction::new(weak_network_server.clone(), Self::on_client_ready_message),
            true,
        );
        self.register_handler::<CommandMessage>(
            SelfMutAction::new(weak_network_server.clone(), Self::on_client_command_message),
            true,
        );
        self.register_handler::<EntityStateMessage>(
            SelfMutAction::new(
                weak_network_server.clone(),
                Self::on_client_entity_state_message,
            ),
            true,
        );
        self.register_handler::<TimeSnapshotMessage>(
            SelfMutAction::new(
                weak_network_server.clone(),
                Self::on_client_time_snapshot_message,
            ),
            false,
        );
        #[allow(static_mut_refs)]
        let weak_network_time = unsafe { NETWORK_TIME.downgrade() };
        self.register_handler::<NetworkPingMessage>(
            SelfMutAction::new(weak_network_time.clone(), NetworkTime::on_server_ping),
            false,
        );
        self.register_handler::<NetworkPongMessage>(
            SelfMutAction::new(weak_network_time.clone(), NetworkTime::on_server_pong),
            false,
        );
    }

    fn on_client_ready_message(
        &mut self,
        connection: RevelArc<Box<NetworkConnectionToClient>>,
        _: ReadyMessage,
        _: TransportChannel,
    ) {
        Self::set_client_ready(connection);
    }

    pub fn set_client_ready(mut connection: RevelArc<Box<NetworkConnectionToClient>>) {
        connection.is_ready = true;
        if connection.identity.upgradable() {
            Self::spawn_observers_for_connection(connection);
        }
    }

    pub fn set_client_not_ready(mut connection: RevelArc<Box<NetworkConnectionToClient>>) {
        connection.is_ready = false;
        connection.remove_from_observings_observers();
        connection.send_message(NotReadyMessage::default(), TransportChannel::Reliable);
    }

    pub fn set_all_clients_not_ready() {
        for conn in Self.connections.values() {
            Self::set_client_not_ready(conn.clone());
        }
    }

    fn spawn_observers_for_connection(mut connection: RevelArc<Box<NetworkConnectionToClient>>) {
        if !connection.is_ready {
            return;
        }

        connection.send_message(
            ObjectSpawnStartedMessage::default(),
            TransportChannel::Reliable,
        );

        // TODO: Spawn observers logic

        connection.send_message(
            ObjectSpawnFinishedMessage::default(),
            TransportChannel::Reliable,
        );
    }

    fn on_client_command_message(
        &mut self,
        connection: RevelArc<Box<NetworkConnectionToClient>>,
        message: CommandMessage,
        channel: TransportChannel,
    ) {
        if !connection.is_ready {
            if channel == TransportChannel::Reliable {
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
                                connection.connection_id
                            );
                        }
                    }
                }
            }
            return;
        }

        match Self.spawned.get(&message.net_id) {
            None => {
                if channel == TransportChannel::Reliable {
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
                                connection.connection_id
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

    fn on_client_entity_state_message(
        &mut self,
        mut connection: RevelArc<Box<NetworkConnectionToClient>>,
        message: EntityStateMessage,
        _: TransportChannel,
    ) {
        match Self.spawned.get(&message.net_id) {
            None => {
                log::warn!(
                    "EntityStateMessage from {} for netId={} without authority.",
                    connection.connection_id,
                    message.net_id
                );
            }
            Some(weak_net_identity) => {
                if let Some(net_identity) = weak_net_identity.get() {
                    if !connection.ptr_eq_weak(&net_identity.connection()) {
                        log::warn!(
                            "EntityStateMessage from {} for {} without authority.",
                            connection.connection_id,
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
                                    connection.disconnect.call(());
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
        &mut self,
        mut connection: RevelArc<Box<NetworkConnectionToClient>>,
        _: TimeSnapshotMessage,
        _: TransportChannel,
    ) {
        let remote_time_stamp = connection.remote_time_stamp;
        connection.on_time_snapshot(TimeSnapshot::new(
            remote_time_stamp,
            NetworkTime.local_time(),
        ))
    }

    fn unpack_and_invoke(
        &mut self,
        mut connection: RevelArc<Box<NetworkConnectionToClient>>,
        reader: &mut NetworkReader,
        channel: TransportChannel,
    ) -> bool {
        if let Some(msg_type) = MessageHandler::unpack_id(reader) {
            let msg_name = match msg_type {
                43708 => "Mirror.ReadyMessage",
                39124 => "Mirror.CommandMessage",
                12339 => "Mirror.EntityStateMessage",
                57097 => "Mirror.TimeSnapshotMessage",
                17487 => "Mirror.NetworkPingMessage",
                27095 => "Mirror.NetworkPongMessage",
                49414 => "Mirror.AddPlayerMessage",
                _ => "Unknown",
            };

            if !vec![17487, 27095, 57097].contains(&msg_type) {
                println!("Received message of type {}", msg_name);
            }

            return match self.message_handlers.get_mut(&msg_type) {
                None => {
                    log::warn!("No handler registered for message type: {}", msg_type);
                    false
                }
                Some(handler) => {
                    connection.last_message_time = NetworkTime.local_time() as f32;
                    handler.invoke(connection, reader, channel);
                    true
                }
            };
        }
        log::warn!(
            "Invalid message header for connection:{}",
            connection.connection_id
        );
        false
    }

    pub fn register_handler<M>(
        &mut self,
        func: SelfMutAction<
            (
                RevelArc<Box<NetworkConnectionToClient>>,
                M,
                TransportChannel,
            ),
            (),
        >,
        require_authentication: bool,
    ) where
        M: NetworkMessage + 'static,
    {
        let message_id = M::get_full_name().hash16();
        println!(
            "register handler for message [{}] {}",
            message_id,
            M::get_full_name()
        );
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
        func: SelfMutAction<
            (
                RevelArc<Box<NetworkConnectionToClient>>,
                M,
                TransportChannel,
            ),
            (),
        >,
        require_authentication: bool,
    ) where
        M: NetworkMessage + 'static,
    {
        let message_id = M::get_full_name().hash16();
        self.message_handlers.insert(
            message_id,
            MessageHandler::new(func, require_authentication),
        );
    }

    pub fn unregister_handler<M>(&mut self)
    where
        M: NetworkMessage + 'static,
    {
        let message_id = M::get_full_name().hash16();
        self.message_handlers.remove(&message_id);
    }

    pub fn shutdown(&mut self) {
        if self.initialized {
            self.disconnect_all();

            TransportManager.active.server_stop();

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
            conn.disconnect.call(());
        }
        self.connections.clear();
    }

    fn cleanup_spawned(&mut self) {
        for (_, identity) in Self.spawned.iter() {
            if let Some(identity) = identity.upgrade() {
                Self::destroy(identity.game_object.clone())
            }
        }
        Self.spawned.clear();
    }

    pub fn add_player_for_connection(
        mut connection: RevelArc<Box<NetworkConnectionToClient>>,
        player: RevelArc<GameObject>,
    ) -> bool {
        match player.try_get_component2::<NetworkIdentity>() {
            None => {
                log::warn!("AddPlayer: player GameObject has no NetworkIdentity. Please add a NetworkIdentity to {}", player.name);
                false
            }
            Some(mut identity) => {
                println!("{}", identity.name());
                connection.identity = identity.downgrade();
                identity.set_client_owner(connection.clone());
                Self::set_client_ready(connection.clone());
                Self::respawn(identity);
                true
            }
        }
    }

    pub fn replace_player_for_connection(
        mut connection: RevelArc<Box<NetworkConnectionToClient>>,
        player: RevelArc<GameObject>,
        replace_player_options: ReplacePlayerOptions,
    ) -> bool {
        match player.try_get_component2::<NetworkIdentity>() {
            None => {
                log::error!("ReplacePlayer: playerGameObject has no NetworkIdentity. Please add a NetworkIdentity to {}",player.name);
                return false;
            }
            Some(mut identity) => {
                if identity.connection().upgradable()
                    && identity.connection().ptr_eq(&connection.downgrade())
                {
                    log::error!("Cannot replace player for connection. New player is already owned by a different connection{}",player.name);
                    return false;
                }

                if let Some(mut previous_player) = connection.identity.upgrade() {
                    connection.identity = identity.downgrade();

                    identity.set_client_owner(connection.clone());

                    Self::spawn_observers_for_connection(connection.clone());

                    Self::respawn(identity.clone());

                    match replace_player_options {
                        ReplacePlayerOptions::KeepAuthority => {
                            Self::send_change_owner_message(
                                previous_player.downgrade(),
                                connection.downgrade(),
                            );
                        }
                        ReplacePlayerOptions::KeepActive => {
                            previous_player.remove_client_authority();
                        }
                        ReplacePlayerOptions::UnSpawn => {
                            Self::un_spawn(previous_player.game_object.clone());
                        }
                        ReplacePlayerOptions::Destroy => {
                            Self::un_spawn(previous_player.game_object.clone());
                        }
                    }

                    return true;
                }
            }
        }
        false
    }

    pub fn send_to_all<T: NetworkMessage>(
        mut message: T,
        channel: TransportChannel,
        send_to_ready_only: bool,
    ) {
        if !Self.active {
            log::warn!("Can not send using NetworkServer.SendToAll<T>(T msg) because NetworkServer is not active");
            return;
        }

        NetworkWriterPool::get_by_closure(|writer| {
            message.serialize(writer);
            let segment = writer.to_vec();

            let max = max_message_size(channel);
            if writer.position > max {
                log::error!(
                    "NetworkServer.SendToAll: message of type {} with a size of {} bytes is larger than the max allowed message size in one batch: {}.\nThe message was dropped, please make it smaller.",
                    T::get_full_name(),
                    writer.position,
                    max
                );
                return;
            }

            let mut count = 0;

            for (_, connection) in Self.connections.iter_mut() {
                if send_to_ready_only && !connection.is_ready {
                    continue;
                }

                count += 1;
                connection.send(&segment, channel);
            }
        })
    }

    fn respawn(identity: RevelArc<Box<NetworkIdentity>>) {
        if let Some(identity_connection) = identity.connection().upgrade() {
            if identity.net_id() == 0 {
                Self::spawn(identity.game_object.clone(), identity_connection.clone())
            } else {
                Self::send_spawn_message(identity.clone(), identity_connection.clone())
            }
        }
    }

    fn send_spawn_message(
        identity: RevelArc<Box<NetworkIdentity>>,
        mut connection: RevelArc<Box<NetworkConnectionToClient>>,
    ) {
        if identity.server_only {
            return;
        }
        let mut owner_writer = RevelArc::new(NetworkWriterPool::get());
        let mut observers_writer = RevelArc::new(NetworkWriterPool::get());

        let is_owner = identity.connection().ptr_eq(&connection.downgrade());
        let is_local = connection.identity.ptr_eq(&identity.downgrade());

        let payload = Self::create_spawn_message_payload(
            is_owner,
            identity.clone(),
            owner_writer.clone(),
            observers_writer.clone(),
        );

        if let Some(identity_game_object) = identity.game_object.upgrade() {
            let mut spawn_message = SpawnMessage::new(
                identity.net_id(),
                is_local,
                is_owner,
                identity.scene_id,
                identity.game_object.get().unwrap().asset_id,
                identity_game_object.transform.local_position,
                identity_game_object.transform.local_rotation,
                identity_game_object.transform.local_scale,
                payload,
            );

            connection.send_message(spawn_message, TransportChannel::Reliable);
        }

        NetworkWriterPool::return_(owner_writer.into_inner());
        NetworkWriterPool::return_(observers_writer.into_inner());
    }

    pub fn create_spawn_message_payload(
        is_owner: bool,
        identity: RevelArc<Box<NetworkIdentity>>,
        owner_writer: RevelArc<NetworkWriter>,
        observers_writer: RevelArc<NetworkWriter>,
    ) -> Vec<u8> {
        if identity.network_behaviours().is_empty() {
            return vec![];
        }

        identity.serialize_server(true, owner_writer.clone(), observers_writer.clone());

        if is_owner {
            owner_writer.to_vec()
        } else {
            observers_writer.to_vec()
        }
    }

    pub fn spawn_objects() -> bool {
        if !Self.active {
            return false;
        }

        let mut identities = vec![];

        for root_game_object in WorldManager::root_game_objects().iter() {
            if let Some(game_object) = root_game_object.get() {
                identities = game_object.get_components::<NetworkIdentity>();
            }
        }

        for identity in identities.iter() {
            if let Some(weak_identity) = identity.downcast::<NetworkIdentity>() {
                if let Some(real_identity) = weak_identity.get() {
                    if real_identity.is_scene_object() && real_identity.net_id() == 0 {
                        if let Some(game_object) = real_identity.game_object.get() {
                            game_object.set_active(true);
                        }
                    }
                }
            }
        }

        for identity in identities.iter() {
            if let Some(weak_identity) = identity.downcast::<NetworkIdentity>() {
                if let Some(real_identity) = weak_identity.get() {
                    if real_identity.is_scene_object()
                        && real_identity.net_id() == 0
                        && Self::valid_parent(real_identity)
                    {
                        if let Some(game_object) = real_identity.game_object.get() {
                            if let Some(identity_connection) = real_identity.connection().upgrade()
                            {
                                Self::spawn(real_identity.game_object.clone(), identity_connection)
                            }
                        }
                    }
                }
            }
        }

        true
    }

    fn spawn(
        game_object: RevelWeak<GameObject>,
        connection: RevelArc<Box<NetworkConnectionToClient>>,
    ) {
        Self::spawn_object(game_object, connection);
    }

    fn spawn_object(
        game_object: RevelWeak<GameObject>,
        connection: RevelArc<Box<NetworkConnectionToClient>>,
    ) {
        if let Some(real_game_object) = game_object.get() {
            if !Self.active {
                log::error!("SpawnObject for {}, NetworkServer is not active. Cannot spawn objects without an active server.", real_game_object.name);
                return;
            }

            match real_game_object.try_get_component::<NetworkIdentity>() {
                None => {
                    log::error!(
                        "SpawnObject {} has no NetworkIdentity. Please add a NetworkIdentity to {}",
                        real_game_object.name,
                        real_game_object.name
                    );
                    return;
                }
                Some(network_identity) => {
                    if let Some(weak_network_identity) =
                        network_identity.downcast::<NetworkIdentity>()
                    {
                        if let Some(identity) = weak_network_identity.get() {
                            if Self.spawned.contains_key(&identity.net_id()) {
                                log::warn!(
                                    "{} [netId={}] was already spawned.",
                                    identity.name(),
                                    identity.net_id()
                                );
                                return;
                            }

                            identity.set_connection(connection.downgrade());

                            if let Some(game_object) = identity.game_object.get() {
                                game_object.set_active(true);
                            }

                            if !identity.is_server && identity.net_id() == 0 {
                                identity.is_server = true;
                                identity.set_net_id(NetworkIdentity::get_next_network_id());

                                Self.spawned
                                    .insert(identity.net_id(), weak_network_identity.clone());

                                identity.on_start_server()
                            }

                            if let Some(identity) = weak_network_identity.upgrade() {
                                Self::rebuild_observers(identity, true)
                            }
                        }
                    }
                }
            }
        }
    }

    fn valid_parent(identity: &mut NetworkIdentity) -> bool {
        if let Some(game_object) = identity.game_object.get() {
            return !game_object.parent.upgradable()
                || (game_object.parent.upgradable()
                    && game_object.parent.get().unwrap().is_active());
        }
        false
    }

    pub fn broadcast() {
        for (_, connection) in Self.connections.iter_mut() {
            if Self::disconnect_if_inactive(connection.clone()) {
                continue;
            }

            if connection.is_ready {
                connection.send_message(TimeSnapshotMessage::new(), TransportChannel::Unreliable);

                Self::broadcast_to_connection(connection.clone());
            }

            connection.update();
        }
    }

    fn disconnect_if_inactive(mut connection: RevelArc<Box<NetworkConnectionToClient>>) -> bool {
        if Self.disconnect_inactive_connections
            && !connection
                .is_alive
                .call((Self.disconnect_inactive_timeout,))
        {
            log::warn!("Disconnecting {} for inactivity!", connection.connection_id);
            connection.disconnect.call(());
            return true;
        }
        false
    }

    pub fn broadcast_to_connection(mut connection: RevelArc<Box<NetworkConnectionToClient>>) {
        let mut connection_clone = connection.clone();
        let mut has_null = false;
        for identity in connection.observing.iter() {
            if identity.upgradable() {
                let serialization =
                    Self::serialize_for_connection(identity.clone(), connection.downgrade());

                match serialization {
                    Some(serialization) => {
                        let message = EntityStateMessage::new(
                            identity.get().unwrap().net_id(),
                            serialization.to_vec(),
                        );
                        connection_clone.send_message(message, TransportChannel::Reliable)
                    }
                    None => {
                        has_null = true;
                        log::warn!(
                            "Found 'null' entry in observing list for connectionId={}. Please call NetworkServer.Destroy to destroy networked objects. Don't use GameObject.Destroy.",
                            connection.connection_id
                        );
                    }
                }
            }
        }

        if has_null {
            connection.observing.retain(|x| x.upgradable());
        }
    }

    fn serialize_for_connection(
        identity: RevelWeak<Box<NetworkIdentity>>,
        connection: RevelWeak<Box<NetworkConnectionToClient>>,
    ) -> Option<RevelArc<NetworkWriter>> {
        if let Some(identity) = identity.get() {
            let serialization = identity.get_server_serialization_at_tick(Time::get_frame_count());

            let owned = identity.connection().ptr_eq(&connection);

            if owned {
                if serialization.owner_writer.position > 0 {
                    return Some(serialization.owner_writer.clone());
                }
            } else {
                if serialization.observers_writer.position > 0 {
                    return Some(serialization.observers_writer.clone());
                }
            }
        }
        None
    }

    pub fn remove_player_for_connection(
        connection: RevelWeak<Box<NetworkConnectionToClient>>,
        remove_options: RemovePlayerOptions,
    ) {
        if let Some(conn) = connection.get() {
            if !conn.identity.upgradable() {
                return;
            }

            match remove_options {
                RemovePlayerOptions::KeepActive => {
                    if let Some(identity) = conn.identity.get() {
                        identity.set_connection(RevelWeak::default());
                        let weak_identity = conn.identity.clone();
                        conn.owned
                            .retain(|owned| !owned.downgrade().ptr_eq(&weak_identity));
                        Self::send_change_owner_message(conn.identity.clone(), connection.clone());
                    }
                }
                RemovePlayerOptions::UnSpawn => {
                    if let Some(identity) = conn.identity.get() {
                        Self::un_spawn(identity.game_object.clone());
                    }
                }
                RemovePlayerOptions::Destroy => {
                    if let Some(identity) = conn.identity.get() {
                        Self::destroy(identity.game_object.clone());
                    }
                }
            }
            conn.identity = RevelWeak::default();
        }
    }
    pub fn destroy(game_object: RevelWeak<GameObject>) {
        if !Self.active {
            log::warn!("NetworkServer.Destroy() called without an active server. Servers can only destroy while active, clients can only ask the server to destroy (for example, with a [Command]), after which the server may decide to destroy the object and broadcast the change to all clients.");
            return;
        }

        if !game_object.upgradable() {
            log::info!("NetworkServer.Destroy(): object is null");
            return;
        }

        if let Some(mut identity) = Self::get_network_identity(game_object.clone()) {
            if identity.scene_id != 0 {
                Self::un_spawn_internal(game_object.clone(), true);
            } else {
                Self::un_spawn_internal(game_object.clone(), false);
                identity.destroy_called = true;

                WorldManager::destroy(&game_object.get().unwrap().id);
            }
        } else {
            log::warn!("NetworkServer.Destroy() called on {} which doesn't have a NetworkIdentity component.",game_object.get().unwrap().name);
            return;
        }
    }

    pub fn rebuild_observers(identity: RevelArc<Box<NetworkIdentity>>, initialize: bool) {
        if let Visibility::ForceShown = identity.visibility {
            Self::rebuild_observers_default(identity, initialize)
        }
    }

    fn rebuild_observers_default(mut identity: RevelArc<Box<NetworkIdentity>>, initialize: bool) {
        if initialize {
            match identity.visibility {
                Visibility::ForceHidden => {
                    if let Some(identity_connection) = identity.connection().upgrade() {
                        identity.add_observer(identity_connection)
                    }
                }
                Visibility::Normal | Visibility::ForceShown => {
                    Self::add_all_ready_server_connections_to_observers(identity.clone());
                }
            }
        }
    }

    fn add_all_ready_server_connections_to_observers(mut identity: RevelArc<Box<NetworkIdentity>>) {
        for (_, connection) in Self.connections.iter_mut() {
            if connection.is_ready {
                identity.add_observer(connection.clone());
            }
        }
    }

    fn get_network_identity(
        game_object: RevelWeak<GameObject>,
    ) -> Option<RevelArc<Box<NetworkIdentity>>> {
        if let Some(identity) = game_object.get() {
            if let Some(identity) = identity.try_get_component::<NetworkIdentity>() {
                if let Some(identity) = identity.downcast::<NetworkIdentity>() {
                    return identity.upgrade();
                }
            }
        }
        None
    }

    pub fn send_change_owner_message(
        identity: RevelWeak<Box<NetworkIdentity>>,
        connection: RevelWeak<Box<NetworkConnectionToClient>>,
    ) {
        if let Some(real_identity) = identity.get() {
            if real_identity.net_id() == 0 || real_identity.server_only {
                return;
            }

            if let Some(real_connection) = connection.get() {
                if real_connection
                    .observing
                    .iter()
                    .filter(|observing| observing.ptr_eq(&identity))
                    .count()
                    > 0
                {
                    return;
                }

                let message = ChangeOwnerMessage::new(
                    real_identity.net_id(),
                    real_identity.connection().ptr_eq(&connection),
                    (real_connection.identity.ptr_eq(&identity)
                        && real_identity.connection().ptr_eq(&connection)),
                );

                real_connection.send_message(message, TransportChannel::Reliable)
            }
        }
    }
    pub fn un_spawn(game_object: RevelWeak<GameObject>) {
        Self::un_spawn_internal(game_object, true)
    }

    pub fn un_spawn_internal(game_object: RevelWeak<GameObject>, reset_state: bool) {
        if !Self.active {
            log::warn!("NetworkServer::un_spawn() called without an active server. \
            Servers can only destroy while active, \
            clients can only ask the server to destroy (for example, with a [Command]), \
            after which the server may decide to destroy the object and broadcast the change to all clients.");
            return;
        }

        if !game_object.upgradable() {
            log::info!("NetworkServer::un_spawn(): object is null");
            return;
        }

        if let Some(mut identity) = Self::get_network_identity(game_object) {
            Self.spawned.remove(&identity.net_id());
            if let Some(conn) = identity.connection().get() {
                conn.remove_owned_object(identity.clone())
            }

            Self::send_to_observers(
                identity.clone(),
                ObjectDestroyMessage::new(identity.net_id()),
            );

            identity.clear_observers();

            identity.on_stop_server();

            if reset_state {
                identity.reset_state();
                if let Some(game_object) = identity.game_object.get() {
                    game_object.set_active(false);
                }
            }
        }
    }

    fn send_to_observers<T: NetworkMessage>(
        identity: RevelArc<Box<NetworkIdentity>>,
        mut message: T,
    ) {
        if identity.observers.len() == 0 {
            return;
        }

        NetworkWriterPool::get_by_closure(|writer| {
            message.serialize(writer);

            if writer.position > max_message_size(TransportChannel::Reliable) {
                log::error!(
                    "NetworkServer.SendToObservers: message of type {} with a size of {} bytes is larger than the max allowed message size in one batch: {}.\nThe message was dropped, please make it smaller.",
                    T::get_full_name(),
                    writer.position,
                    max_message_size(TransportChannel::Reliable)
                );
                return;
            }

            let segment = writer.to_vec();
            for observer in identity.observers.values() {
                if let Some(observer) = observer.get() {
                    observer.send(&segment, TransportChannel::Reliable);
                }
            }
        });
    }

    pub fn hide_for_connection(
        identity: RevelArc<Box<NetworkIdentity>>,
        mut connection: RevelArc<Box<NetworkConnectionToClient>>,
    ) {
        let message = ObjectHideMessage::new(identity.net_id());
        connection.send_message(message, TransportChannel::Reliable)
    }
}

// 生命周期
impl NetworkServer {
    pub(crate) fn network_early_update() {
        if Self.active {
            Self.early_update_duration.begin();
            Self.full_update_duration.begin();
        }

        TransportManager.active.server_early_update();

        for (_, connection) in Self.connections.iter_mut() {
            connection.update_time_interpolation()
        }

        if Self.active {
            Self.early_update_duration.end();
        }
    }
    pub(crate) fn network_late_update() {
        if Self.active {
            Self.late_update_duration.begin();
        }

        let send_interval_elapsed = AccurateInterval::elapsed(
            NetworkTime.local_time(),
            Self.send_interval(),
            &mut Self.late_send_time,
        );

        if send_interval_elapsed {
            Self::broadcast();
        }

        TransportManager.active.server_late_update();

        if Self.active {
            Self.actual_tick_rate_counter += 1;

            if NetworkTime.local_time() >= Self.actual_tick_rate_start {
                let elapsed = NetworkTime.local_time() - Self.actual_tick_rate_start;
                Self.actual_tick_rate = (Self.actual_tick_rate_counter as f64 / elapsed) as i32;
                Self.actual_tick_rate_start = NetworkTime.local_time();
                Self.actual_tick_rate_counter = 0;
            }

            Self.late_update_duration.end();
            Self.full_update_duration.end();
        }
    }

    pub fn show_for_connection(
        identity: RevelArc<Box<NetworkIdentity>>,
        conn: RevelArc<Box<NetworkConnectionToClient>>,
    ) {
        // TODO
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
