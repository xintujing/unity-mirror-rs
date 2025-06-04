use crate::commons::action::SelfMutAction;
use crate::commons::revel_arc::RevelArc;
use crate::commons::revel_weak::RevelWeak;
use crate::metadata_settings::metadata::Metadata;
use crate::metadata_settings::mirror::metadata_network_manager::{
    MetadataNetworkManager, MetadataNetworkManagerWrapper,
};
use crate::mirror::authenticator::basic_authenticator::BasicAuthenticatorRequestMessage;
use crate::mirror::messages::add_player_message::AddPlayerMessage;
use crate::mirror::messages::network_pong_message::NetworkPongMessage;
use crate::mirror::messages::ready_message::ReadyMessage;
use crate::mirror::messages::scene_message::{SceneMessage, SceneOperation};
use crate::mirror::network_manager_factory::NetworkManagerFactory;
use crate::mirror::snapshot_interpolation::snapshot_interpolation_settings::SnapshotInterpolationSettings;
use crate::mirror::transport::{Transport, TransportChannel, TransportError, TransportManager};
use crate::mirror::{
    network_manager_trait, Authenticator, AuthenticatorFactory, NetworkConnection,
    NetworkRoomManager, NetworkServer, TNetworkManager,
};
use crate::transports::kcp2k2_transport::Kcp2kTransport;
use crate::unity_engine::{
    GameObject, LoadSceneMode, MonoBehaviour, Time, Transform, WorldManager,
};
use kcp2k_rust::kcp2k_config::Kcp2KConfig;
use once_cell::sync::Lazy;
use rand::Rng;
use std::any::{Any, TypeId};
use std::cell::UnsafeCell;
use std::collections::HashMap;
use std::ops::Deref;
use unity_mirror_macro::{namespace, network_manager, virtual_trait, NetworkManagerFactory};

static mut NETWORK_MANAGER: Lazy<Vec<RevelWeak<Box<dyn TNetworkManager>>>> =
    Lazy::new(|| Vec::default());
static mut NETWORK_MANAGER_MAPPING: Lazy<HashMap<TypeId, usize>> = Lazy::new(|| HashMap::default());

impl NetworkManager {
    pub fn is_instance() -> bool {
        #[allow(static_mut_refs)]
        unsafe {
            !NETWORK_MANAGER.is_empty()
        }
    }
    pub fn singleton<T: TNetworkManager + 'static>(f: fn(&mut T)) {
        #[allow(static_mut_refs)]
        unsafe {
            let type_id = TypeId::of::<T>();
            if let Some(index) = NETWORK_MANAGER_MAPPING.get(&type_id) {
                if let Some(network_manager) = NETWORK_MANAGER.get(*index) {
                    if let Some(weak) = network_manager.downcast::<T>() {
                        if let Some(real) = weak.get() {
                            f(real)
                        }
                    }
                }
            }
        }
    }
}

#[derive(Default)]
pub enum PlayerSpawnMethod {
    Random = 0,
    #[default]
    RoundRobin = 1,
}

#[derive(Default)]
pub enum ConnectionQualityMethod {
    #[default]
    Simple = 0, // simple estimation based on rtt and jitter
    Pragmatic = 1, // based on snapshot interpolation adjustment
}

#[namespace(prefix = "Mirror")]
#[network_manager]
#[virtual_trait({
    on_start_server(&mut self);
    on_stop_server(&mut self);
    on_server_connect(& mut self, connection: RevelArc<NetworkConnection>);

    on_server_change_scene(&mut self,scene_name: String);
    on_server_scene_changed(&mut self,scene_name: String);

    on_server_disconnect(&self, connection: RevelArc<NetworkConnection>);
    on_server_error(&self, connection: RevelArc<NetworkConnection>, error: TransportError, reason: String);
    on_server_transport_exception(&self, connection: RevelArc<NetworkConnection>, error: Box<dyn std::error::Error>);
})]
pub struct NetworkManager {
    pub dont_destroy_on_load: bool,

    pub send_rate: i32,

    pub start_scene: String,
    pub offline_scene: String,
    pub online_scene: String,
    network_scene_name: String,

    pub offline_scene_load_delay: f32,
    pub player_prefab: String,
    pub auto_create_player: bool,
    pub exceptions_disconnect: bool,
    pub snapshot_settings: SnapshotInterpolationSettings,
    pub evaluation_method: ConnectionQualityMethod,
    pub evaluation_interval: f32,
    pub time_interpolation_gui: bool,
    pub spawn_prefabs: Vec<String>,

    max_connections: i32,
    disconnect_inactive_connections: bool,
    disconnect_inactive_timeout: f32,

    player_spawn_method: PlayerSpawnMethod,
    start_position_index: usize,
    pub start_positions: HashMap<String, Vec<Transform>>,

    pub authenticator: Option<Box<dyn Authenticator>>,
    transport: Option<RevelArc<Box<dyn Transport>>>,

    pub on_client_scene_changed: SelfMutAction<(), ()>,
}

impl NetworkManager {
    pub fn get_network_scene_name(&self) -> String {
        self.network_scene_name.clone()
    }
    fn set_network_scene_name(&mut self, name: &str) {
        self.network_scene_name = name.to_string();
    }
}

impl MonoBehaviour for NetworkManager {
    fn awake(&mut self) {
        if !self.initialize_singleton() {
            return;
        }

        self.apply_configuration();

        self.network_scene_name = self.offline_scene.clone();

        WorldManager::set_scene_loaded(SelfMutAction::new(self.weak.clone(), Self::on_scene_loaded))
    }

    fn start(&mut self) {
        self.start_server()
    }
    fn update(&mut self) {
        self.on_client_scene_changed.call(());
        // if let Some(ref mut on_client_scene_changed) = self.on_client_scene_changed {
        //     on_client_scene_changed.invoke(());
        // }

        // println!("Mirror: NetworkManager Update");
        if let Some(virtual_trait) = self.virtual_trait.get() {
            virtual_trait.on_start_server();
        } else {
            // default code
            // println!("Mirror: NetworkManager Default virtual_trait");
        }
    }

    fn late_update(&mut self) {
        self.update_scene()
    }
}

impl NetworkManagerInitialize for NetworkManager {
    fn initialize(&mut self, metadata: &MetadataNetworkManagerWrapper) {
        let config = metadata.get::<MetadataNetworkManager>();

        self.dont_destroy_on_load = config.dont_destroy_on_load;
        self.send_rate = config.send_rate;
        self.start_scene = config.start_scene.asset_path.clone();
        if let Some(offline_scene) = &config.offline_scene {
            self.offline_scene = offline_scene.asset_path.clone()
        }
        if let Some(online_scene) = &config.online_scene {
            self.online_scene = online_scene.asset_path.clone()
        }

        self.offline_scene_load_delay = config.offline_scene_load_delay;
        self.player_prefab = config.player_prefab.asset_path.clone();
        self.auto_create_player = config.auto_create_player;
        self.exceptions_disconnect = config.exceptions_disconnect;
        self.snapshot_settings = SnapshotInterpolationSettings {
            buffer_time_multiplier: config.snapshot_settings.buffer_time_multiplier,
            buffer_limit: config.snapshot_settings.buffer_limit,
            catchup_negative_threshold: config.snapshot_settings.catchup_negative_threshold,
            catchup_positive_threshold: config.snapshot_settings.catchup_positive_threshold,
            catchup_speed: config.snapshot_settings.catchup_speed,
            slowdown_speed: config.snapshot_settings.slowdown_speed,
            drift_ema_duration: config.snapshot_settings.drift_ema_duration,
            dynamic_adjustment: config.snapshot_settings.dynamic_adjustment,
            dynamic_adjustment_tolerance: config.snapshot_settings.dynamic_adjustment_tolerance,
            delivery_time_ema_duration: config.snapshot_settings.delivery_time_ema_duration,
        };

        // self.evaluation_method = config.evaluation_method.clone().into();
        // self.evaluation_interval = config.evaluation_interval;
        // self.time_interpolation_gui = config.time_interpolation_gui;

        self.spawn_prefabs = config
            .spawn_prefabs
            .iter()
            .map(|prefab| prefab.asset_path.clone())
            .collect::<Vec<_>>();
        self.max_connections = 100;
        // self.disconnect_inactive_connections = config.disconnect_inactive_connections;
        self.disconnect_inactive_timeout = 60f32;

        self.authenticator = Some(AuthenticatorFactory::create(
            "Mirror.Authenticators.BasicAuthenticator",
        ));

        self.transport = Some(RevelArc::new(Kcp2kTransport::new(Some(Kcp2KConfig {
            ..Kcp2KConfig::default()
        }))));

        self.send_rate = 60;
    }
}

impl NetworkManager {
    fn start_server(&mut self) {
        if NetworkServer.active {
            log::warn!("Server already started.");
            return;
        }

        self.setup_server();

        if let Some(virtual_trait) = self.virtual_trait.get() {
            virtual_trait.on_start_server()
        }
        if self.is_server_online_scene_change_needed() {
            let online_scene = self.online_scene.clone();
            self.server_change_scene(&online_scene);
        } else {
            NetworkServer::spawn_objects();
        }
    }
    fn setup_server(&mut self) {
        self.initialize_singleton();

        NetworkServer.disconnect_inactive_connections = self.disconnect_inactive_connections;
        NetworkServer.disconnect_inactive_timeout = self.disconnect_inactive_timeout;
        NetworkServer.exceptions_disconnect = self.exceptions_disconnect;

        if let Some(ref mut authenticator) = self.authenticator {
            authenticator.on_start_server();
            authenticator.set_on_server_authenticated(SelfMutAction::new(
                self.weak.clone(),
                Self::on_server_authenticated,
            ))
        }

        self.configure_headless_frame_rate();

        NetworkServer.listen(self.max_connections);

        self.register_server_messages()
    }

    pub fn init(prefab_path: &str) {
        if Self::is_instance() {
            return;
        }
        let metadata = Metadata::get_network_manager(&prefab_path).unwrap();
        let full_name = metadata.get_final_full_name();

        let mut arc_game_object = RevelArc::new(GameObject::default());

        let instances =
            NetworkManagerFactory::create(&full_name, arc_game_object.downgrade(), metadata);

        let instances = instances
            .into_iter()
            .map(|(instance, type_id)| {
                let instance = unsafe {
                    &*(&instance as *const dyn Any as *const RevelArc<Box<dyn MonoBehaviour>>)
                };
                (instance.clone(), type_id)
            })
            .collect::<Vec<_>>();

        arc_game_object.add_component(instances);
        WorldManager::dont_destroy_object(arc_game_object);
    }

    fn initialize_singleton(&self) -> bool {
        if Self::is_instance() {
            return true;
        }

        if let Some(game_object) = self.game_object.get() {
            if let Some(network_manager_vec) = game_object.components.get(0) {
                for (index, network_manager) in network_manager_vec.iter().enumerate() {
                    if let Some(t_network_manager) = network_manager
                        .downgrade()
                        .parallel::<Box<dyn TNetworkManager>>()
                    {
                        #[allow(static_mut_refs)]
                        unsafe {
                            NETWORK_MANAGER.push(t_network_manager.clone());
                            NETWORK_MANAGER_MAPPING
                                .insert(t_network_manager.get().unwrap().self_type_id(), index);
                        }
                    }
                }
            } else {
                log::error!("No NetworkManager component on NetworkManager prefab.");
                return false;
            }

            // if let Some(component) = game_object.components.get(0).unwrap().last() {
            //     if let Some(t_network_manager) =
            //         component.downgrade().parallel::<Box<dyn TNetworkManager>>()
            //     {
            //         unsafe {
            //             *NETWORK_MANAGER = t_network_manager.clone();
            //         }
            //     }
            // }
        }

        if self.transport.is_none() {
            log::error!("No Transport on Network Manager...add a transport and assign it.");
        }

        if let Some(transport) = &self.transport {
            TransportManager.active = transport.downgrade()
        }

        true
    }

    fn is_server_online_scene_change_needed(&self) -> bool {
        !self.online_scene.is_empty()
            && self.network_scene_name != self.online_scene
            && self.online_scene != self.offline_scene
    }

    fn server_change_scene(&mut self, scene_name: &str) {
        if scene_name.is_empty() {
            log::error!("ServerChangeScene empty scene name");
            return;
        }

        if NetworkServer.is_loading_scene && scene_name == self.network_scene_name {
            log::error!("Scene change is already in progress for {}", scene_name);
            return;
        }
        if !NetworkServer.active && scene_name != self.offline_scene {
            log::error!("ServerChangeScene can only be called on an active server.");
            return;
        }

        NetworkServer::set_all_clients_not_ready();
        self.set_network_scene_name(scene_name);

        self.virtual_trait
            .get()
            .map(|virtual_trait| virtual_trait.on_server_change_scene(scene_name.to_string()));

        NetworkServer.is_loading_scene = true;

        WorldManager::load_scene(scene_name, LoadSceneMode::Single);

        if NetworkServer.active {
            let message = SceneMessage::new(scene_name.to_string(), SceneOperation::Normal, false);
            NetworkServer::send_to_all(message, TransportChannel::Reliable, false);
        }

        self.start_position_index = 0;
        // self.start_positions.Clear();
    }

    fn on_server_authenticated(&mut self, mut connection: RevelArc<NetworkConnection>) {
        connection.is_authenticated = true;

        if self.network_scene_name != "" && self.network_scene_name != self.offline_scene {
            let mut message = SceneMessage::new(
                self.network_scene_name.clone(),
                SceneOperation::Normal,
                false,
            );
            connection.send_message(&mut message, TransportChannel::Reliable);
        }

        if let Some(virtual_trait) = self.virtual_trait.get() {
            virtual_trait.on_server_connect(connection.clone())
        }
    }

    fn configure_headless_frame_rate(&self) {
        Time::set_frame_rate(self.send_rate as u16)
    }

    fn register_server_messages(&self) {
        NetworkServer.on_connected_event =
            SelfMutAction::new(self.weak.clone(), Self::on_server_connect_internal);
        NetworkServer.on_disconnected_event =
            SelfMutAction::new(self.weak.clone(), Self::on_server_disconnect);
        NetworkServer.on_error_event = SelfMutAction::new(self.weak.clone(), Self::on_server_error);
        NetworkServer.on_transport_exception_event =
            SelfMutAction::new(self.weak.clone(), Self::on_server_transport_exception);

        NetworkServer.register_handler::<AddPlayerMessage>(
            SelfMutAction::new(self.weak.clone(), Self::on_server_add_player_internal),
            false,
        );
        NetworkServer.replace_handler::<ReadyMessage>(
            SelfMutAction::new(self.weak.clone(), Self::on_server_ready_message_internal),
            false,
        );
    }

    fn on_server_connect_internal(&mut self, connection: RevelArc<NetworkConnection>) {
        if let Some(authenticator) = &self.authenticator {
            authenticator.on_server_authenticate(connection)
        } else {
            self.on_server_authenticated(connection)
        }
    }
    fn on_server_disconnect(&mut self, connection: RevelArc<NetworkConnection>) {
        if let Some(virtual_trait) = self.virtual_trait.get() {
            virtual_trait.on_server_disconnect(connection)
        }
    }
    fn on_server_error(
        &mut self,
        connection: RevelArc<NetworkConnection>,
        error: TransportError,
        reason: String,
    ) {
        if let Some(virtual_trait) = self.virtual_trait.get() {
            virtual_trait.on_server_error(connection, error, reason)
        }
    }
    fn on_server_transport_exception(
        &mut self,
        connection: RevelArc<NetworkConnection>,
        error: Box<dyn std::error::Error>,
    ) {
        if let Some(virtual_trait) = self.virtual_trait.get() {
            virtual_trait.on_server_transport_exception(connection, error)
        }
    }
    fn on_server_add_player_internal(
        &mut self,
        mut connection: RevelArc<NetworkConnection>,
        message: AddPlayerMessage,
        _: TransportChannel,
    ) {
        if self.auto_create_player && self.player_prefab.is_empty() {
            log::error!("The PlayerPrefab is empty on the NetworkManager. Please setup a PlayerPrefab object.");
            return;
        }
        if self.auto_create_player && Metadata::get_prefab(&self.player_prefab).is_none() {
            log::error!("The PlayerPrefab does not have a NetworkIdentity. Please add a NetworkIdentity to the player prefab.");
            return;
        }

        if connection.identity.upgradable() {
            log::error!("There is already a player for this connection.");
            return;
        }
        self.on_server_add_player(connection)
    }

    fn on_server_add_player(&mut self, connection: RevelArc<NetworkConnection>) {
        if let Some(player_prefab) = Metadata::get_prefab(&self.player_prefab) {
            let mut player = GameObject::instance(player_prefab);
            if let Some(start_position) = self.get_start_position() {
                player.transform = RevelArc::new(start_position);
            }
            player.name = format!("{} [connId={}]", player.name, connection.id);
            NetworkServer::add_player_for_connection(connection, player);
        }
    }

    fn get_start_position(&mut self) -> Option<Transform> {
        let current_scene = self.network_scene_name.clone();

        if !self.start_positions.contains_key(&current_scene) {
            return None;
        }

        let start_positions = self.start_positions[&current_scene].clone();

        if start_positions.len() == 0 {
            return None;
        }

        let index = match self.player_spawn_method {
            PlayerSpawnMethod::Random => rand::rng().random_range(0..=start_positions.len()),
            PlayerSpawnMethod::RoundRobin => self.start_position_index,
        };

        if let Some(start_position) = start_positions.get(index) {
            self.start_position_index = (self.start_position_index + 1) % start_positions.len();
            return Some(start_position.clone());
        }

        None
    }
    fn on_server_ready_message_internal(
        &mut self,
        mut connection: RevelArc<NetworkConnection>,
        _message: ReadyMessage,
        _: TransportChannel,
    ) {
        self.on_server_ready(connection);
    }

    fn on_server_ready(&self, mut connection: RevelArc<NetworkConnection>) {
        if !connection.identity.upgradable() {}
        NetworkServer::set_client_ready(connection);
    }

    fn on_scene_loaded(&mut self, _: String, mode: LoadSceneMode) {
        if let LoadSceneMode::Additive = mode {
            if NetworkServer.active {
                NetworkServer::spawn_objects();
            }
        }
    }

    fn apply_configuration(&self) {
        NetworkServer.tick_rate = self.send_rate as u32;
    }

    fn update_scene(&self) {
        if !WorldManager.loading() {
            self.finish_load_scene()
        }
    }

    fn finish_load_scene(&self) {
        NetworkServer.is_loading_scene = false;

        NetworkServer::spawn_objects();
        if let Some(virtual_trait) = self.virtual_trait.get() {
            let network_scene_name = self.network_scene_name.clone();
            virtual_trait.on_server_scene_changed(network_scene_name)
        }
    }
}

// impl NetworkManager {
//     /// 验证网络管理器的状态。
//     /// 确保网络管理器处于有效状态。
//     fn on_validate(&self) -> bool {
//         todo!()
//     }
//
//     /// 重置网络管理器。
//     /// 仅在组件被添加或用户重置组件时调用。
//     /// 这就是为什么我们验证这些仅在添加 NetworkManager 时需要验证的内容。
//     /// 如果我们在 OnValidate() 中执行它，那么每次值更改时都会运行。
//     fn reset(&mut self) {
//         todo!()
//     }
//
//     /// 唤醒网络管理器。
//     /// 不允许碰撞销毁的第二个实例继续。
//     /// 如果无法初始化单例，则返回。
//     /// 在 Awake 中应用配置。
//     /// 设置 networkSceneName 以防止客户端连接到服务器失败时重新加载场景。
//     /// 设置 OnSceneLoaded 回调。
//     fn awake(&self) {
//         todo!()
//     }
//
//     /// 启动网络管理器。
//     /// 自动启动无头服务器或客户端。
//     /// 我们不能在 Awake 中执行此操作，因为 Awake 是用于初始化的，
//     /// 并且某些传输可能在 Start 之前尚未准备好。
//     /// 在编辑器中自动启动对于调试很有用，因此可以通过 editorAutoStart 启用。
//     fn start(&mut self) -> Result<(), String> {
//         todo!()
//     }
//
//     /// 更新网络管理器状态。
//     /// 在每次 Update() 中应用一些 NetworkServer/Client 配置。
//     /// 以避免两个数据源的冲突。
//     /// 修复了 NetworkServer.sendRate 从未设置的问题，因为从未调用 NetworkManager.StartServer。
//     /// 如果 NM 存在，则所有公开的设置应始终应用。
//     fn update(&mut self, delta_time: f32) {
//         todo!()
//     }
//
//     /// 延迟更新网络管理器状态。
//     /// 在主更新之后执行额外的更新逻辑。
//     /// 如果 loadingSceneAsync 不为 null 且已完成，则调用 FinishLoadScene。
//     fn late_update(&mut self, delta_time: f32) {
//         todo!()
//     }
//
//     /// 检查是否需要服务器在线场景切换。
//     /// 如果需要切换到在线场景，则返回 true。
//     /// 仅在请求的在线场景不为空且尚未加载时更改场景。
//     fn is_server_online_scene_change_needed(&self) -> bool {
//         todo!()
//     }
//
//     /// 应用网络配置。
//     /// 根据提供的配置字符串更新网络设置。
//     /// NetworkServer.tickRate = sendRate;
//     /// NetworkClient.snapshotSettings = snapshotSettings;
//     /// NetworkClient.connectionQualityInterval = evaluationInterval;
//     /// NetworkClient.connectionQualityMethod = evaluationMethod;
//     fn apply_configuration(&mut self, config: &str) -> Result<(), String> {
//         todo!()
//     }
//
//     /// 设置服务器。
//     /// 配置服务器地址和端口。
//     /// 在初始化任何内容之前应用设置。
//     /// NetworkServer.disconnectInactiveConnections = disconnectInactiveConnections;
//     /// NetworkServer.disconnectInactiveTimeout = disconnectInactiveTimeout;
//     /// NetworkServer.exceptionsDisconnect = exceptionsDisconnect;
//     fn setup_server(&mut self, address: &str, port: u16) -> Result<(), String> {
//         todo!()
//     }
//
//     /// 启动服务器。
//     /// 开始监听并接受连接。
//     /// StartServer 本质上是异步的（不会立即完成）。
//     /// 这里是它的作用：
//     /// - 监听
//     /// - 如果 onlineScene:
//     ///   - LoadSceneAsync
//     ///   - ...
//     ///   - FinishLoadSceneServerOnly
//     ///     - SpawnObjects
//     /// - 否则:
//     ///   - SpawnObjects
//     fn start_server(&mut self) -> Result<(), String> {
//         todo!()
//     }
//
//     /// 停止服务器。
//     /// 停止服务器的运行并清理资源。
//     /// 在更改场景之前设置离线模式，以便 FinishStartScene 不认为我们需要初始化任何内容。
//     fn stop_server(&mut self) {
//         todo!()
//     }
//
//     /// 应用程序退出时的处理。
//     /// 在应用程序关闭时执行清理操作。
//     /// 首先停止客户端（我们希望将退出数据包发送到服务器，而不是等待超时）。
//     /// 然后停止服务器（用于正确的主机模式停止）。
//     fn on_application_quit(&self) {
//         todo!()
//     }
//
//     /// 配置无头模式的帧率。
//     /// 如果是无头模式，则设置服务器或客户端的目标帧率。
//     fn configure_headless_frame_rate(&mut self, frame_rate: u32) {
//         todo!()
//     }
//
//     /// 初始化单例。
//     /// 确保网络管理器的唯一实例。
//     /// 如果 singleton 不为 null 且等于当前实例，则返回 true。
//     /// 如果 dontDestroyOnLoad 为 true，则将对象设置为场景根。
//     /// 如果传输未分配，则尝试从当前对象获取传输。
//     fn initialize_singleton(&mut self) -> Result<(), String> {
//         todo!()
//     }
//
//     /// 注册服务器消息。
//     /// 注册服务器需要处理的消息类型。
//     /// NetworkServer.OnConnectedEvent = OnServerConnectInternal;
//     /// NetworkServer.OnDisconnectedEvent = OnServerDisconnect;
//     /// NetworkServer.OnErrorEvent = OnServerError;
//     /// NetworkServer.OnTransportExceptionEvent = OnServerTransportException;
//     fn register_server_messages(&mut self, messages: Vec<String>) {
//         todo!()
//     }
//
//     /// 注册起始位置。
//     /// 将一个位置注册为玩家的出生点。
//     /// 通过 NetworkStartPosition 组件自动完成，但也可以通过用户脚本代码手动完成。
//     /// 按层次顺序重新排序列表，以便轮询生成使用起始位置。
//     fn register_start_position(&mut self, position: (f32, f32, f32)) {
//         todo!()
//     }
//
//     /// 注销起始位置。
//     /// 从玩家出生点列表中移除一个位置。
//     /// 通过 NetworkStartPosition::OnDestroy 自动完成。
//     fn unregister_start_position(&mut self, position: (f32, f32, f32)) {
//         todo!()
//     }
//
//     /// 获取起始位置。
//     /// 返回一个可用的玩家出生点。
//     /// 首先移除所有无效的变换。
//     /// 如果没有起始位置，则返回 null。
//     /// 如果玩家生成方法是随机的，则返回随机位置。
//     /// 否则返回轮询位置。
//     fn get_start_position(&self) -> Option<(f32, f32, f32)> {
//         todo!()
//     }
//
//     /// 更新场景。
//     /// 如果 loadingSceneAsync 不为 null 且已完成，则调用 FinishLoadScene。
//     fn update_scene(&mut self, delta_time: f32) {
//         todo!()
//     }
//
//     /// 销毁网络管理器。
//     /// 清理网络管理器的资源。
//     fn on_destroy(&mut self) {
//         todo!()
//     }
//
//     /// 重置静态变量。<br>
//     /// 清理或重新初始化静态数据。<br>
//     /// 调用 StopHost() 如果我们有一个 singleton。<br>
//     /// 重置所有静态变量。<br>
//     /// 清除 startPositions。<br>
//     /// 将 startPositionIndex 设置为 0。<br>
//     /// 将 clientReadyConnection 设置为 null。<br>
//     /// 将 loadingSceneAsync 设置为 null。<br>
//     /// 将 networkSceneName 设置为空字符串。<br>
//     /// 最后将 singleton 设置为 null。
//     fn reset_statics(&mut self) {
//         todo!()
//     }
// }
