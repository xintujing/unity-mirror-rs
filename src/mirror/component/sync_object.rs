use crate::mirror::components::network_behaviour::{NetworkBehaviour, SyncDirection};
use crate::mirror::network_reader::NetworkReader;
use crate::mirror::network_writer::NetworkWriter;
use std::fmt::{Debug, Formatter};

pub struct BaseSyncObject {
    on_dirty: Box<dyn Fn() -> u64>,
    is_writable: Box<dyn Fn() -> bool>,
    is_recording: Box<dyn Fn() -> bool>,
}

// #[va]
// impl BaseSyncObject {
//     pub fn on_dirty() {}
//     pub fn is_writable() -> bool {
//         true
//     }
//     pub fn is_recording() -> bool {
//         true
//     }
// }

pub trait SyncObject: Debug + Default {
    fn component_id(&self) -> &'static str;
    fn on_dirty(&mut self, index: usize) {
        if let Some(mut state) = NetworkBehaviour::state_mut(self.component_id()) {
            state.sync_object_dirty_bit |= 1 << index;
        }
    }
    fn is_writable(&self) -> bool {
        if let Some(state) = NetworkBehaviour::state(&self.component_id()) {
            return state.sync_direction == SyncDirection::ServerToClient;
        }
        log::error!("NetworkBehaviour are not active.");
        false
    }
    fn is_recording(&self) -> bool {
        if let Some(state) = NetworkBehaviour::state(&self.component_id()) {
            return state.identity.get_observers_len() > 0;
        }
        false
    }
    fn on_serialize_all(&self, writer: &mut NetworkWriter);
    fn on_serialize_delta(&self, writer: &mut NetworkWriter);
    fn on_deserialize_all(&mut self, reader: &mut NetworkReader);
    fn on_deserialize_delta(&mut self, reader: &mut NetworkReader);
}
