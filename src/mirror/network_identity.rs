use crate::commons::action::SelfMutAction;
use crate::commons::revel_arc::RevelArc;
use crate::commons::revel_weak::RevelWeak;
use crate::metadata_settings::mirror::metadata_network_identity::{
    MetadataNetworkIdentity, MetadataNetworkIdentityWrapper,
};
use crate::mirror::NetworkBehaviourFactory;
use crate::mirror::NetworkReader;
use crate::mirror::NetworkWriter;
use crate::mirror::NetworkWriterPool;
use crate::mirror::{
    NetworkConnectionToClient, NetworkServer, RemoteCallType, RemoteProcedureCalls, SyncDirection,
    SyncMode, TNetworkBehaviour,
};
use crate::unity_engine::MonoBehaviour;
use crate::unity_engine::MonoBehaviourFactory;
use crate::unity_engine::{GameObject, WorldManager};
use lazy_static::lazy_static;
use once_cell::sync::Lazy;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering::SeqCst;
use unity_mirror_macro_rs::namespace;

#[ctor::ctor]
fn static_init() {
    MonoBehaviourFactory::register::<NetworkIdentity>(|weak_game_object, metadata| {
        let wrapper = metadata
            .as_any()
            .downcast_ref::<MetadataNetworkIdentityWrapper>()
            .unwrap();

        let identity =
            NetworkIdentity::instance(weak_game_object, wrapper.get::<MetadataNetworkIdentity>());
        let type_id = identity.type_id();

        let arc_identity = RevelArc::new(Box::new(identity) as Box<dyn MonoBehaviour>);

        if let Some(weak_identity) = arc_identity.downgrade().downcast::<NetworkIdentity>() {
            if let Some(mut identity) = weak_identity.upgrade() {
                identity.self_weak = weak_identity.clone();
            }
        }

        vec![(arc_identity, type_id)]
    });
}

lazy_static! {
    static ref NEXT_NETWORK_ID: AtomicU32 = AtomicU32::new(1);
}
#[allow(unused)]
static mut SCENE_IDS: Lazy<HashMap<u64, RevelWeak<NetworkIdentity>>> = Lazy::new(|| HashMap::new());

#[allow(unused)]
pub(crate) trait IntoNum {
    fn to_u32(&self) -> u32;
    fn to_u64(&self) -> u64;
}

#[allow(unused)]
impl IntoNum for str {
    fn to_u32(&self) -> u32 {
        self.parse::<u32>().unwrap_or(0)
    }

    fn to_u64(&self) -> u64 {
        self.parse::<u64>().unwrap_or(0)
    }
}

#[allow(unused)]
#[derive(Eq, PartialEq, Default)]
pub enum Visibility {
    #[default]
    Normal,
    ForceHidden,
    ForceShown,
}

impl Into<Visibility>
for crate::metadata_settings::mirror::metadata_network_identity::MetadataVisibility
{
    fn into(self) -> Visibility {
        match self {
            crate::metadata_settings::mirror::metadata_network_identity::MetadataVisibility::Default => Visibility::Normal,
            crate::metadata_settings::mirror::metadata_network_identity::MetadataVisibility::ForceHidden => Visibility::ForceHidden,
            crate::metadata_settings::mirror::metadata_network_identity::MetadataVisibility::ForceShown => Visibility::ForceShown,
        }
    }
}

#[derive(Default)]
pub struct NetworkIdentitySerialization {
    pub tick: u64,
    pub owner_writer: RevelArc<NetworkWriter>,
    pub observers_writer: RevelArc<NetworkWriter>,
}

impl NetworkIdentitySerialization {
    pub fn reset_writers(&mut self) {
        self.owner_writer.reset();
        self.observers_writer.reset();
    }
}

#[namespace(prefix = "Mirror")]
#[derive(Default)]
pub struct NetworkIdentity {
    pub self_weak: RevelWeak<Box<NetworkIdentity>>,
    pub game_object: RevelWeak<GameObject>,

    net_id: u32,
    component_mapping: HashMap<TypeId, Vec<usize>>,
    network_behaviours: Vec<Vec<RevelWeak<Box<dyn TNetworkBehaviour>>>>,
    connection: RevelWeak<Box<NetworkConnectionToClient>>,

    pub is_server: bool,
    pub server_only: bool,
    pub is_client: bool,
    pub is_owned: bool,

    pub scene_id: u64,
    _asset_id: u32,
    pub destroy_called: bool,
    pub visibility: Visibility,

    owner_payload: Vec<u8>,
    observers_payload: Vec<u8>,

    pub(crate) observers: HashMap<u64, RevelWeak<Box<NetworkConnectionToClient>>>,
    last_serialization: RevelArc<NetworkIdentitySerialization>,

    spawned_from_instantiate: bool,
    has_spawned: bool,
    had_authority: bool,

    client_authority_callback: SelfMutAction<(RevelArc<Box<NetworkConnectionToClient>>, RevelArc<Box<NetworkIdentity>>, bool), ()>,
}

impl PartialEq<Self> for NetworkIdentity {
    fn eq(&self, other: &Self) -> bool {
        self.net_id == other.net_id
    }
}

impl Hash for NetworkIdentity {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.net_id.hash(state);
    }
}

impl Eq for NetworkIdentity {}

impl MonoBehaviour for NetworkIdentity {
    fn awake(&mut self) {
        self.initialize_network_behaviours();

        if self.has_spawned {
            log::error!(
                "{} has already spawned. Don't call Instantiate for NetworkIdentities that were in the scene since the beginning (aka scene objects).  Otherwise the client won't know which object to use for a SpawnSceneObject message.",
                self.name()
            );
            self.spawned_from_instantiate = true;
            WorldManager::destroy(&self.game_object.get().unwrap().id);
        }
        self.has_spawned = true;
    }
    fn update(&mut self) {
        // println!("Mirror: NetworkIdentity Update");
    }
    fn on_destroy(&mut self) {
        // println!("Mirror: NetworkIdentity Destroyed");
    }
}

impl NetworkIdentity {
    const MAX_NETWORK_BEHAVIOURS: usize = 64;
    pub fn get_next_network_id() -> u32 {
        let curr = NEXT_NETWORK_ID.load(SeqCst);
        NEXT_NETWORK_ID.store(curr + 1, SeqCst);
        curr
    }

    pub fn reset_server_statics() {
        NEXT_NETWORK_ID.store(1, SeqCst);
    }

    pub fn remove_observer(&mut self, conn: RevelArc<Box<NetworkConnectionToClient>>) {
        self.observers.remove(&conn.connection_id);
    }

    pub fn set_client_owner(&mut self, arc: RevelArc<Box<NetworkConnectionToClient>>) {
        self.connection = arc.downgrade();
    }

    pub fn remove_client_authority(&mut self) {
        if !self.is_server {
            log::error!(
                "RemoveClientAuthority can only be called on the server for spawned objects."
            );
            return;
        }

        if let Some(connection) = self.connection.upgrade() {
            if connection.identity.ptr_eq(&self.self_weak) {
                log::error!("RemoveClientAuthority cannot remove authority for a player object");
                return;
            }
        }

        if let (Some(arc_connection), Some(arc_self)) =
            (self.connection.upgrade(), self.self_weak.upgrade())
        {
            self.client_authority_callback
                .call((arc_connection.clone(), arc_self.clone(), false));

            self.set_connection(RevelWeak::new());

            NetworkServer::send_change_owner_message(
                arc_self.downgrade(),
                arc_connection.downgrade(),
            );
        }
    }

    fn initialize_network_behaviours(&mut self) {
        for (index, network_behaviour_chain) in self.network_behaviours.iter().enumerate() {
            if let Some(network_behaviour) = network_behaviour_chain.last() {
                if let Some(mut network_behaviour) = network_behaviour.upgrade() {
                    network_behaviour.initialize(index as u8, self.self_weak.clone());
                }
            }
        }
    }

    pub fn handle_remote_call(
        &self,
        component_index: u8,
        function_hash: u16,
        remote_call_type: RemoteCallType,
        reader: &mut NetworkReader,
        sender_connection: RevelArc<Box<NetworkConnectionToClient>>,
    ) {
        if component_index >= self.component_mapping.len() as u8 {
            log::warn!(
                "NetworkIdentity: handle_remote_call: component_index {} out of bounds for identity with net_id {}",
                component_index,
                self.net_id
            );
            return;
        }
        let invoke_component_chain = self.network_behaviours[component_index as usize].clone();

        if !RemoteProcedureCalls.invoke(
            function_hash,
            &remote_call_type,
            reader,
            invoke_component_chain,
            sender_connection,
        ) {
            log::error!(
                "Found no receiver for incoming {:?} [{}] on {}, the server and client should have the same NetworkBehaviour instances [netId={}].",
                remote_call_type,
                function_hash,
                self.name(),
                self.net_id
            );
        }
    }

    pub fn get_server_serialization_at_tick(&mut self, tick: u64) -> RevelArc<NetworkIdentitySerialization> {
        if self.last_serialization.tick != tick {
            self.last_serialization.reset_writers();

            self.serialize_server(
                false,
                self.last_serialization.owner_writer.clone(),
                self.last_serialization.observers_writer.clone(),
            );
            self.last_serialization.tick = tick;
        }

        self.last_serialization.clone()
    }

    fn validate_components(&self) {}
}

// 序列化相关
impl NetworkIdentity {
    // ServerDirtyMasks
    fn server_dirty_masks(&self, initial_state: bool) -> (u64, u64) {
        let mut owner_mask = 0u64;
        let mut observer_mask = 0u64;

        for (i, network_behaviour_chain) in self.network_behaviours.iter().enumerate() {
            if let Some(network_behaviour) = network_behaviour_chain.last().and_then(|x| x.get()) {
                let nth_bit = 1u64 << (i as u8);
                let dirty = network_behaviour.is_dirty();

                if initial_state || (dirty && (network_behaviour.get_sync_direction().eq(&SyncDirection::ServerToClient))) {
                    owner_mask |= nth_bit;
                }

                if (network_behaviour.get_sync_mode().eq(&SyncMode::Observers)) && (initial_state || dirty) {
                    observer_mask |= nth_bit;
                }
            }
        }
        (owner_mask, observer_mask)
    }

    fn is_dirty(&self, mask: u64, index: u8) -> bool {
        (mask & (1u64 << index)) != 0
    }

    pub(crate) fn serialize_server(
        &self,
        initial_state: bool,
        mut owner_writer: RevelArc<NetworkWriter>,
        mut observers_writer: RevelArc<NetworkWriter>,
    ) {
        let (owner_mask, observer_mask) = self.server_dirty_masks(initial_state);

        if owner_mask != 0 {
            owner_writer.write_blittable_compress(owner_mask);
        }
        if observer_mask != 0 {
            observers_writer.write_blittable_compress(observer_mask);
        }

        if (owner_mask | observer_mask) != 0 {
            for (network_behaviour_i, network_behaviour_chain) in self.network_behaviours.iter().enumerate() {
                let owner_dirty = self.is_dirty(owner_mask, network_behaviour_i as u8);
                let observers_dirty = self.is_dirty(observer_mask, network_behaviour_i as u8);

                if owner_dirty || observers_dirty {
                    NetworkWriterPool::get_by_closure(|writer| {
                        // serialize
                        if let Some(last) = network_behaviour_chain.last() {
                            if let Some(comp) = last.get() {
                                let header_position = writer.position;
                                writer.write_byte(0);
                                let content_position = writer.position;

                                comp.on_serialize(writer, initial_state);

                                let end_position = writer.position;
                                writer.position = header_position;
                                let size = (end_position - content_position) as i32;
                                let safety = (size & 0xFF) as u8;
                                writer.write_byte(safety);
                                writer.position = end_position;
                            }
                        }
                        if owner_dirty {
                            owner_writer.write_slice(writer.to_slice(), 0, writer.position);
                        }
                        if observers_dirty {
                            observers_writer.write_slice(writer.to_slice(), 0, writer.position);
                        }
                    });

                    if !initial_state {
                        if let Some(last) = network_behaviour_chain.last() {
                            if let Some(comp) = last.get() {
                                comp.clear_all_dirty_bits();
                            }
                        }
                    }
                }
            }
        }
    }

    pub(crate) fn deserialize_server(&self, _reader: &mut NetworkReader) -> bool {
        true
    }

    pub fn clear_observers(&mut self) {
        for (_, observer) in self.observers.iter_mut() {
            if let (Some(self_arc), Some(mut real_observer)) =
                (self.self_weak.upgrade(), observer.upgrade())
            {
                real_observer.remove_from_observing(self_arc, true)
            }
        }
        self.observers.clear();
    }

    pub fn reset_state(&mut self) {
        self.has_spawned = false;
        self.is_client = false;
        self.is_server = false;

        self.is_owned = false;

        self.notify_authority();

        self.net_id = 0;

        self.connection = RevelWeak::default();

        self.clear_observers();
    }

    fn notify_authority(&mut self) {
        if !self.had_authority && self.is_owned {
            self.on_start_authority();
        }
        if self.had_authority && !self.is_owned {
            self.on_stop_authority();
        }
        self.had_authority = self.is_owned;
    }

    fn on_start_authority(&self) {
        for network_behaviour in self.network_behaviours.iter() {
            if let Some(network_behaviour) = network_behaviour.last() {
                if let Some(mut network_behaviour) = network_behaviour.upgrade() {
                    network_behaviour.on_start_authority();
                }
            }
        }
    }
    fn on_stop_authority(&self) {
        for network_behaviour in self.network_behaviours.iter() {
            if let Some(network_behaviour) = network_behaviour.last() {
                if let Some(mut network_behaviour) = network_behaviour.upgrade() {
                    network_behaviour.on_stop_authority();
                }
            }
        }
    }
}

// 实例化
impl NetworkIdentity {
    fn instance(
        weak_game_object: RevelWeak<GameObject>,
        settings: &MetadataNetworkIdentity,
    ) -> Self {
        let mut identity = Self {
            game_object: weak_game_object.clone(),
            ..Default::default()
        };
        if let Some(game_object) = weak_game_object.get() {
            for metadata_network_behaviour_wrapper in settings.network_behaviours.iter() {
                let final_full_name = metadata_network_behaviour_wrapper.get_final_full_name();
                let network_behaviours = NetworkBehaviourFactory::create(
                    &final_full_name,
                    weak_game_object.clone(),
                    metadata_network_behaviour_wrapper,
                );

                let mut weak_network_behaviours = Vec::new();

                let index = identity.network_behaviours.len();
                for (behaviour, type_id) in network_behaviours.iter() {
                    let tmp_behaviour = behaviour.clone();
                    weak_network_behaviours.push(unsafe {
                        (&*(&tmp_behaviour as *const dyn Any as *const RevelWeak<Box<dyn TNetworkBehaviour>>)).clone()
                    });

                    if !identity.component_mapping.contains_key(&type_id) {
                        identity.component_mapping.insert(*type_id, vec![index]);
                    } else {
                        if let Some(mapping) = identity.component_mapping.get_mut(&type_id) {
                            mapping.push(index);
                        };
                    }
                }
                identity.network_behaviours.push(weak_network_behaviours);
                game_object.add_component(network_behaviours);
            }
        }

        identity
    }

    pub fn name(&self) -> String {
        if let Some(game_object) = self.game_object.upgrade() {
            game_object.name.clone()
        } else {
            "Unknown".to_string()
        }
    }

    pub fn net_id(&self) -> u32 {
        self.net_id
    }

    pub fn set_net_id(&mut self, net_id: u32) {
        self.net_id = net_id;
    }

    pub fn connection(&self) -> RevelWeak<Box<NetworkConnectionToClient>> {
        self.connection.clone()
    }

    pub fn set_connection(&mut self, connections: RevelWeak<Box<NetworkConnectionToClient>>) {
        if let Some(conn) = connections.get() {
            if let Some(identity) = self.self_weak.upgrade() {
                conn.remove_owned_object(identity.clone());
            }
        }
        self.connection = connections;
        if let Some(conn) = self.connection.get() {
            if let Some(identity) = self.self_weak.upgrade() {
                conn.add_owned_object(identity.clone());
            }
        }
    }

    pub fn network_behaviours(&self) -> &Vec<Vec<RevelWeak<Box<dyn TNetworkBehaviour>>>> {
        &self.network_behaviours
    }

    pub fn is_server_only(&self) -> bool {
        self.is_server && !self.is_client
    }

    pub fn is_client_only(&self) -> bool {
        !self.is_server && self.is_client
    }

    pub fn is_scene_object(&self) -> bool {
        self.scene_id != 0
    }
}

impl NetworkIdentity {
    pub fn on_start_server(&self) {
        for network_behaviour in self.network_behaviours.iter() {
            if let Some(weak_network_behaviour) = network_behaviour.last() {
                if let Some(real_network_behaviour) = weak_network_behaviour.get() {
                    real_network_behaviour.on_start_server();
                }
            }
        }
    }

    pub fn on_stop_server(&self) {
        for network_behaviour in self.network_behaviours.iter() {
            if let Some(weak_network_behaviour) = network_behaviour.last() {
                if let Some(real_network_behaviour) = weak_network_behaviour.get() {
                    real_network_behaviour.on_stop_server();
                }
            }
        }
    }

    pub fn add_observer(&mut self, mut conn: RevelArc<Box<NetworkConnectionToClient>>) {
        if self.observers.contains_key(&conn.connection_id) {
            return;
        }

        if self.observers.is_empty() {
            self.clear_all_components_dirty_bits()
        }

        self.observers.insert(conn.connection_id, conn.downgrade());
        if let Some(self_arc) = self.self_weak.upgrade() {
            log::debug!("has network_behaviours: {}", self.network_behaviours.len());
            conn.add_to_observing(self_arc)
        }
    }

    fn clear_all_components_dirty_bits(&self) {
        for component in self.network_behaviours.iter() {
            if let Some(component) = component.last() {
                if let Some(mut component) = component.upgrade() {
                    component.clear_all_dirty_bits();
                }
            }
        }
    }
}
