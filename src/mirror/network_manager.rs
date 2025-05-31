use crate::commons::action::SelfMutAction;
use crate::commons::revel_arc::RevelArc;
use crate::commons::revel_weak::RevelWeak;
use crate::metadata_settings::metadata::Metadata;
use crate::metadata_settings::mirror::metadata_network_manager::MetadataNetworkManagerWrapper;
use crate::mirror::network_manager_factory::NetworkManagerFactory;
use crate::mirror::{network_manager_trait, Authenticator, AuthenticatorFactory, NetworkConnection, NetworkServer};
use crate::unity_engine::{GameObject, MonoBehaviour, Time, Transform, WorldManager};
use once_cell::sync::Lazy;
use std::any::Any;
use rand::Rng;
use unity_mirror_macro::{callbacks, namespace, network_manager, NetworkManagerFactory};
use crate::mirror::authenticator::basic_authenticator::BasicAuthenticatorRequestMessage;
use crate::mirror::messages::add_player_message::AddPlayerMessage;
use crate::mirror::messages::network_pong_message::NetworkPongMessage;
use crate::mirror::messages::ready_message::ReadyMessage;
use crate::mirror::messages::scene_message::{SceneMessage, SceneOperation};
use crate::mirror::transport::{TransportChannel, TransportError};

static mut NETWORK_MANAGER: Lazy<RevelWeak<Box<dyn network_manager_trait::TNetworkManager>>> =
    Lazy::new(|| RevelWeak::default());

static mut NETWORK_MANAGER_PREFAB_PATH: Option<String> = None;

impl NetworkManager {
    pub fn set_network_manager_prefab_path(path: String) {
        #[allow(static_mut_refs)]
        unsafe {
            NETWORK_MANAGER_PREFAB_PATH = Some(path);
        }
    }

    pub fn get_network_manager_prefab_path() -> Option<String> {
        #[allow(static_mut_refs)]
        unsafe {
            NETWORK_MANAGER_PREFAB_PATH.clone()
        }
    }
}
impl NetworkManager {
    pub fn is_instance(&self) -> bool {
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

#[network_manager]
#[namespace(prefix = "Mirror")]
#[derive(NetworkManagerFactory)]
#[callbacks({
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
    pub authenticator: Option<Box<dyn Authenticator>>,

    // Action Begin
    pub on_client_scene_changed: SelfMutAction<(), ()>,
    // Action End

    start_scene: String,
    online_scene: String,
    offline_scene: String,

    network_scene_name: String,

    disconnect_inactive_connections: bool,
    disconnect_inactive_timeout: f32,
    exceptions_disconnect: bool,

    max_connections: i32,

    send_rate: i32,

    player_spawn_method: PlayerSpawnMethod,
    start_position_index: i32,
    start_positions: Vec<Transform>,

    player_prefab: String,
    auto_create_player: bool,
}

impl NetworkManager {
    fn set_network_scene_name(&mut self, name: &str) {
        self.network_scene_name = name.to_string();
    }
}

impl MonoBehaviour for NetworkManager {
    fn awake(&mut self) {
        // println!("Mirror: NetworkManager Awake");
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
        if let Some(callbacks) = self.callbacks.get() {
            callbacks.on_start_server();
        } else {
            // default code
            // println!("Mirror: NetworkManager Default callbacks");
        }
    }
}

impl NetworkManagerInitialize for NetworkManager {
    fn initialize(&mut self, metadata: &MetadataNetworkManagerWrapper) {
        // let config = metadata.get::<MetadataNetworkManager>();
        // config.authenticator.initialize(self);

        self.authenticator = Some(AuthenticatorFactory::create(
            "Mirror.Authenticators.BasicAuthenticator",
        ));
    }
}

impl NetworkManager {
    fn start() {}
    fn start_server(&mut self) {
        if NetworkServer.active {
            log::warn!("Server already started.");
            return;
        }

        self.setup_server();

        if let Some(callbacks) = self.callbacks.get() {
            callbacks.on_start_server()
        }

        if self.is_server_online_scene_change_needed() {
            self.server_change_scene(&self.online_scene);
        } else {
            NetworkServer::spawn_objects()
        }
    }
    fn setup_server(&self) {
        self.initialize_singleton();

        NetworkServer.disconnect_inactive_connections = self.disconnect_inactive_connections;
        NetworkServer.disconnect_inactive_timeout = self.disconnect_inactive_timeout;
        NetworkServer.exceptions_disconnect = self.exceptions_disconnect;

        if let Some(ref mut authenticator) = self.authenticator {
            authenticator.on_start_server();
            authenticator.set_on_server_authenticated(SelfMutAction::new(self.weak.clone(), Self::on_server_authenticated))
        }

        self.configure_headless_frame_rate();

        NetworkServer.listen(self.max_connections);

        self.register_server_messages()
    }

    fn _new() {
        if let Some(prefab_path) = Self::get_network_manager_prefab_path() {
            let metadata = Metadata::get_network_manager(&prefab_path).unwrap();
            let full_name = metadata.get_final_full_name();

            let mut arc_game_object = RevelArc::new(GameObject::default());

            let instances =
                NetworkManagerFactory::create(&full_name, arc_game_object.downgrade(), metadata);

            if let Some((instance, last_type_id)) = instances.last() {
                #[allow(static_mut_refs)]
                unsafe {
                    *NETWORK_MANAGER = instance.downgrade();
                }
            }
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
    }

    fn initialize_singleton(&self) -> bool {
        if self.is_instance() {
            return true;
        }
        Self::_new();
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
            log::error!("Scene change is already in progress for {}",scene_name);
            return;
        }
        if !NetworkServer.active && scene_name != self.offline_scene {
            log::error!("ServerChangeScene can only be called on an active server.");
            return;
        }

        NetworkServer::set_all_clients_not_ready();
        self.set_network_scene_name(scene_name);

        self.callbacks.get().map(|callbacks| {
            callbacks.on_server_change_scene(scene_name.to_string())
        });

        NetworkServer.is_loading_scene = true;

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
            let mut message = SceneMessage::new(self.network_scene_name.clone(), SceneOperation::Normal, false);
            connection.send_message(&mut message, TransportChannel::Reliable);
        }

        if let Some(callbacks) = self.callbacks.get() {
            callbacks.on_server_connect(connection.clone())
        }
    }


    fn configure_headless_frame_rate(&self) {
        Time::set_frame_rate(self.send_rate as u16)
    }

    fn register_server_messages(&self) {
        NetworkServer.on_connected_event = SelfMutAction::new(self.weak.clone(), Self::on_server_connect_internal);
        NetworkServer.on_disconnected_event = SelfMutAction::new(self.weak.clone(), Self::on_server_disconnect);
        NetworkServer.on_error_event = SelfMutAction::new(self.weak.clone(), Self::on_server_error);
        NetworkServer.on_transport_exception_event = SelfMutAction::new(self.weak.clone(), Self::on_server_transport_exception);

        NetworkServer.register_handler::<AddPlayerMessage>(|c, m, _| {
            let weak_self = self.weak.clone();
            Self::on_server_add_player_internal(&mut **(weak_self.upgrade().unwrap()), c, m)
        }, false);
        NetworkServer.replace_handler::<ReadyMessage>(|c, m, _| {
            let weak_self = self.weak.clone();
            Self::on_server_ready_message_internal(&mut **(weak_self.upgrade().unwrap()), c, m)
        }, false);
    }

    fn on_server_connect_internal(&mut self, connection: RevelArc<NetworkConnection>) {
        if let Some(authenticator) = &self.authenticator {
            authenticator.on_server_authenticate(connection)
        } else {
            self.on_server_authenticated(connection)
        }
    }
    fn on_server_disconnect(&mut self, connection: RevelArc<NetworkConnection>) {
        if let Some(callbacks) = self.callbacks.get() {
            callbacks.on_server_disconnect(connection)
        }
    }
    fn on_server_error(&mut self, connection: RevelArc<NetworkConnection>, error: TransportError, reason: String) {
        if let Some(callbacks) = self.callbacks.get() {
            callbacks.on_server_error(connection, error, reason)
        }
    }
    fn on_server_transport_exception(&mut self, connection: RevelArc<NetworkConnection>, error: Box<dyn std::error::Error>) {
        if let Some(callbacks) = self.callbacks.get() {
            callbacks.on_server_transport_exception(connection, error)
        }
    }
    fn on_server_add_player_internal(
        &mut self,
        mut connection: RevelArc<NetworkConnection>,
        message: AddPlayerMessage,
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
            let player = GameObject::instance(player_prefab);
            if let Some(start_position) = self.get_start_position() {
                player.transform = RevelArc::new(start_position);
            }
            player.name = format!("{} [connId={}]", player.name, connection.id);
            NetworkServer::add_player_for_connection(connection, player);
        }
    }

    fn get_start_position(&mut self) -> Option<Transform> {
        if self.start_positions.is_empty() {
            return None;
        }

        Some(match &self.player_spawn_method {
            PlayerSpawnMethod::Random => {
                let index = rand::rng().random_range(0..self.start_positions.len());
                self.start_positions[index].clone()
            }
            PlayerSpawnMethod::RoundRobin => {
                let start_position = self.start_positions[self.start_position_index];
                self.start_position_index = (self.start_position_index + 1) % self.start_positions.len() as i32;
                start_position
            }
        })
    }
    fn on_server_ready_message_internal(
        &self,
        mut connection: RevelArc<NetworkConnection>,
        _message: ReadyMessage,
    ) {
        self.on_server_ready(connection);
    }

    fn on_server_ready(&self, mut connection: RevelArc<NetworkConnection>) {
        if !connection.identity.upgradable() {}
        NetworkServer::set_client_ready(connection);
    }
}
