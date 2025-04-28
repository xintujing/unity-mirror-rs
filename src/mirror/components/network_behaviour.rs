use crate::commons::reference::Reference;
use std::sync::{Arc, Mutex};

use crate::metadata_settings::mirror::network_behaviours::metadata_network_behaviour::{
    MetadataNetworkBehaviour, MetadataNetworkBehaviourWrapper, MetadataSyncDirection,
    MetadataSyncMode,
};

use crate::mirror::component::component_basic::ComponentBasic;
use crate::mirror::component::component_lifespan::ComponentLifespan;
use crate::mirror::component::state::StateInitialize;
use crate::mirror::component::sync_object::SyncObject;
use crate::mirror::network_connection::NetworkConnection;
use crate::mirror::network_identity::NetworkIdentity;
use crate::unity::game_object::GameObject;
use unity_mirror_rs_macro::{component, state};
use String;

#[derive(Default, Debug, Clone)]
pub enum SyncDirection {
    #[default]
    ServerToClient,
    ClientToServer,
}

impl Into<SyncDirection> for MetadataSyncDirection {
    fn into(self) -> SyncDirection {
        match &self {
            MetadataSyncDirection::ServerToClient => SyncDirection::ServerToClient,
            MetadataSyncDirection::ClientToServer => SyncDirection::ClientToServer,
        }
    }
}

#[derive(Default, Debug, Clone)]
pub enum SyncMode {
    #[default]
    Observers,
    Owner,
}

impl Into<SyncMode> for MetadataSyncMode {
    fn into(self) -> SyncMode {
        match &self {
            MetadataSyncMode::Observers => SyncMode::Observers,
            MetadataSyncMode::Owner => SyncMode::Owner,
        }
    }
}

#[allow(unused)]
#[state]
pub struct NetworkBehaviourState {
    pub game_object_ref: Reference<GameObject>,
    pub identity: NetworkIdentity,
    pub index: u8,
    pub connection_ref: Reference<NetworkConnection>,

    // 同步方向
    pub sync_direction: SyncDirection,
    // 同步模式
    pub sync_mode: SyncMode,
    // 同步间隔
    pub sync_interval: f64,
    // 同步脏位
    pub sync_var_dirty_bit: u64,
    pub sync_object_dirty_bit: u64,
    // 最后同步时间
    pub last_sync_time: f64,

    pub sync_objects: Vec<Arc<Mutex<Box<dyn SyncObject>>>>,
}

impl StateInitialize for NetworkBehaviourState {
    fn initialize(&mut self, settings: &MetadataNetworkBehaviourWrapper)
    where
        Self: Sized,
    {
        let settings = settings.get::<MetadataNetworkBehaviour>();
        self.sync_direction = settings.sync_direction.clone().into();
        self.sync_mode = settings.sync_mode.clone().into();
        self.sync_interval = settings.sync_interval;
    }
}

impl NetworkBehaviourStateOnChangeCallback for NetworkBehaviourState {}

#[component(namespace("Mirror"), state(NetworkBehaviourState))]
pub struct NetworkBehaviour;

impl NetworkBehaviour {
    pub fn error_correction(size: usize, safety: u8) -> usize {
        let cleared = size & 0xFFFFFF00;
        cleared | (safety as usize)
    }
}

impl ComponentLifespan for NetworkBehaviour {}
//
// impl NetworkBehaviour {
//     pub fn init_sync_object<T: SyncObject>(&self, mut sync_object: T) -> Reference<T> {
//         Self::state(&self.id()).map(|state| state.sync_objects.len());
//
//         let mut index = 0;
//         let mut sync_direction: SyncDirection::ClientToServer;
//         let mut observers_len = 0;
//
//         if let Some(state) = Self::state(&self.id()) {
//             index = state.sync_objects.len();
//             sync_direction = state.sync_direction.clone();
//             observers_len = state.identity.get_observers_len();
//         }
//         sync_object.set_on_dirty(Box::new(|| 1u64 << index));
//         sync_object.set_is_writable(Box::new(|| sync_direction == SyncDirection::ServerToClient));
//         sync_object.set_is_recording(Box::new(|| observers_len > 0));
//
//         let sync_object = Arc::new(Mutex::new(Box::new(sync_object)));
//         let weak = Arc::downgrade(&sync_object);
//
//         if let Some(mut state) = Self::state_mut(&self.id()) {
//             state.sync_objects.push(sync_object);
//         }
//
//         Reference::new(weak)
//     }
//     pub(crate) fn serialize_objects_all(id: &str, writer: &mut NetworkWriter) {
//         if let Some(state) = Self::state(&id) {
//             for sync_object in state.sync_objects.iter() {
//                 if let Ok(sync_object) = sync_object.lock() {
//                     sync_object.on_serialize_all(writer);
//                 }
//             }
//         }
//     }
//     pub(crate) fn serialize_objects_delta(id: &ComponentId, writer: &mut NetworkWriter) {
//         if let Some(state) = Self::state(&id) {
//             writer.write_blittable::<u64>(state.sync_object_dirty_bit);
//             for (i, sync_object) in state.sync_objects.iter().enumerate() {
//                 if (state.sync_object_dirty_bit & (1u64 << i)) != 0 {
//                     if let Ok(sync_object) = sync_object.lock() {
//                         sync_object.on_serialize_delta(writer);
//                     }
//                 }
//             }
//         }
//     }
//     pub(crate) fn deserialize_objects_all(id: &ComponentId, reader: &mut NetworkReader) {
//         if let Some(mut state) = Self::state(&id) {
//             for sync_object in state.sync_objects.iter() {
//                 if let Ok(mut sync_object) = sync_object.lock() {
//                     sync_object.on_deserialize_all(reader);
//                 }
//             }
//         }
//     }
//     pub(crate) fn deserialize_objects_delta(id: &ComponentId, reader: &mut NetworkReader) {
//         if let Some(mut state) = Self::state(&id) {
//             let dirty_bit = reader.read_blittable::<u64>();
//             for (i, sync_object) in state.sync_objects.iter().enumerate() {
//                 if (dirty_bit & (1u64 << i)) != 0 {
//                     if let Ok(mut sync_object) = sync_object.lock() {
//                         sync_object.on_deserialize_delta(reader);
//                     }
//                 }
//             }
//         }
//     }
// }

////
