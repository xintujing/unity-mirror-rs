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
    network_manager_trait, Authenticator, AuthenticatorFactory, NetworkConnection, NetworkServer,
    TNetworkManager,
};
use crate::transports::kcp2k2_transport::Kcp2kTransport;
use crate::unity_engine::{
    GameObject, LoadSceneMode, MonoBehaviour, Time, Transform, WorldManager,
};
use kcp2k_rust::kcp2k_config::Kcp2KConfig;
use once_cell::sync::Lazy;
use rand::Rng;
use std::any::Any;
use std::cell::UnsafeCell;
use std::collections::HashMap;
use std::ops::Deref;
use unity_mirror_macro::{namespace, network_manager, virtual_trait, NetworkManagerFactory};

static mut NETWORK_MANAGER: Lazy<RevelWeak<Box<dyn network_manager_trait::TNetworkManager>>> =
    Lazy::new(|| RevelWeak::default());

impl NetworkManager {
    pub fn is_instance() -> bool {
        #[allow(static_mut_refs)]
        unsafe {
            NETWORK_MANAGER.upgradable()
        }
    }
    pub fn singleton<T: network_manager_trait::TNetworkManager + 'static>(f: fn(&mut T)) {
        #[allow(static_mut_refs)]
        unsafe {
            if let Some(weak) = NETWORK_MANAGER.downcast::<T>() {
                if let Some(real) = weak.get() {
                    f(real)
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
            if let Some(component) = game_object.components.get(0).unwrap().last() {
                if let Some(t_network_manager) =
                    component.downgrade().parallel::<Box<dyn TNetworkManager>>()
                {
                    unsafe {
                        *NETWORK_MANAGER = t_network_manager.clone();
                    }
                }
            }
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
            NetworkServer::send_to_all(message)
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
