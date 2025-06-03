use crate::commons::revel_arc::RevelArc;
use crate::commons::revel_weak::RevelWeak;
use crate::metadata_settings::mirror::metadata_network_identity::{
    MetadataNetworkIdentity, MetadataNetworkIdentityWrapper,
};
use crate::mirror::network_behaviour_factory::NetworkBehaviourFactory;
use crate::mirror::network_connection::NetworkConnection;
use crate::mirror::network_reader::NetworkReader;
use crate::mirror::network_writer::NetworkWriter;
use crate::mirror::network_writer_pool::NetworkWriterPool;
use crate::mirror::{
    RemoteCallType, RemoteProcedureCalls, SyncDirection, SyncMode, TNetworkBehaviour,
};
use crate::unity_engine::GameObject;
use crate::unity_engine::MonoBehaviour;
use crate::unity_engine::MonoBehaviourFactory;
use lazy_static::lazy_static;
use once_cell::sync::Lazy;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering::SeqCst;
use unity_mirror_macro::namespace;

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
        vec![(Box::new(identity), type_id)]
    });
}

lazy_static! {
    static ref NEXT_NETWORK_ID: AtomicU32 = AtomicU32::new(1);
}
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

#[namespace(prefix = "Mirror")]
#[derive(Default)]
pub struct NetworkIdentity {
    net_id: u32,
    component_mapping: HashMap<TypeId, Vec<usize>>,
    network_behaviours: Vec<Vec<RevelWeak<Box<dyn TNetworkBehaviour>>>>,
    connection: RevelWeak<NetworkConnection>,

    pub is_server: bool,
    pub server_only: bool,
    pub is_client: bool,
    pub is_owned: bool,

    pub scene_id: u64,
    _asset_id: u32,
    destroy_called: bool,
    pub visibility: Visibility,

    owner_payload: Vec<u8>,
    observers_payload: Vec<u8>,

    pub(crate) observers: Vec<RevelWeak<NetworkConnection>>,
}

impl MonoBehaviour for NetworkIdentity {
    fn awake(&mut self) {
        // println!("Mirror: NetworkIdentity Awake");
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

    pub fn handle_remote_call(
        &self,
        component_index: u8,
        function_hash: u16,
        remote_call_type: RemoteCallType,
        reader: &mut NetworkReader,
        sender_connection: RevelArc<NetworkConnection>,
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
}

// 序列化相关
impl NetworkIdentity {
    // ServerDirtyMasks
    fn server_dirty_masks(&self, initial_state: bool) -> (u64, u64) {
        let mut owner_mask = 0u64;
        let mut observer_mask = 0u64;

        for (i, network_behaviour_chain) in self.network_behaviours.iter().enumerate() {
            if let Some(network_behaviour) = network_behaviour_chain.first().and_then(|x| x.get()) {
                let nth_bit = 1u64 << (i as u8);
                let dirty = network_behaviour.is_dirty();

                if initial_state
                    || (dirty
                    && (network_behaviour
                    .get_sync_direction()
                    .eq(&SyncDirection::ServerToClient)))
                {
                    owner_mask |= nth_bit;
                }

                if (network_behaviour.get_sync_mod().eq(&SyncMode::Observers))
                    && (initial_state || dirty)
                {
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
        owner_writer: &mut NetworkWriter,
        observers_writer: &mut NetworkWriter,
    ) {
        let (owner_mask, observer_mask) = self.server_dirty_masks(initial_state);

        if owner_mask != 0 {
            owner_writer.write_blittable_compress(observer_mask);
        }
        if observer_mask != 0 {
            observers_writer.write_blittable_compress(owner_mask);
        }

        if (owner_mask | observer_mask) != 0 {
            for (network_behaviour_i, network_behaviour_chain) in
                self.network_behaviours.iter().enumerate()
            {
                let owner_dirty = self.is_dirty(owner_mask, network_behaviour_i as u8);
                let observers_dirty = self.is_dirty(observer_mask, network_behaviour_i as u8);

                if owner_dirty || observers_dirty {
                    NetworkWriterPool::get_return(|writer| {
                        // serialize
                        if let Some(last) = network_behaviour_chain.last() {
                            if let Some(comp) = last.get() {
                                comp.on_serialize(writer, initial_state);
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

    pub(crate) fn deserialize_server(&self, reader: &mut NetworkReader) -> bool {
        true
    }
}

// 实例化
impl NetworkIdentity {
    fn instance(
        weak_game_object: RevelWeak<GameObject>,
        settings: &MetadataNetworkIdentity,
    ) -> Self {
        let mut identity = Self {
            net_id: 12366,
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

                let index = identity.network_behaviours.len();
                for (_, type_id) in network_behaviours.iter() {
                    if !identity.component_mapping.contains_key(&type_id) {
                        identity.component_mapping.insert(*type_id, vec![index]);
                    } else {
                        if let Some(mapping) = identity.component_mapping.get_mut(&type_id) {
                            mapping.push(index);
                        };
                    }
                }
                game_object.add_component(network_behaviours);

                //
                // // let (mono_behaviour, type_id) = NetworkBehaviourFactory::create(
                // //     &final_full_name,
                // //     weak_game_object.clone(),
                // //     metadata_network_behaviour_wrapper,
                // // );
                // // let arc_network_behaviour = RevelArc::new(mono_behaviour);
                //
                // identity
                //     .network_behaviours
                //     .push(arc_network_behaviour.downgrade());
                // // identity.network_behaviours.push(WeakRwLock::new(&arc_network_behaviour));
                // let index = identity.network_behaviours.len() - 1;
                // if !identity.component_mapping.contains_key(&type_id) {
                //     identity.component_mapping.insert(type_id, vec![index]);
                // } else {
                //     if let Some(mapping) = identity.component_mapping.get_mut(&type_id) {
                //         mapping.push(index);
                //     };
                // }
                //
                // game_object.add_component(vec![(arc_network_behaviour, type_id)]);
            }
        }

        // println!("Mirror: NetworkIdentity Instance");

        identity
    }

    pub fn name(&self) -> String {
        "NetworkIdentity".to_string()
    }

    pub fn net_id(&self) -> u32 {
        self.net_id
    }

    pub fn connection(&self) -> RevelWeak<NetworkConnection> {
        self.connection.clone()
    }

    pub fn set_connection(&mut self, connections: RevelWeak<NetworkConnection>) {
        self.connection = connections;
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
}
