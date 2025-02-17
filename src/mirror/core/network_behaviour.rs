use crate::mirror::core::network_reader::NetworkReader;
use crate::mirror::core::network_writer::NetworkWriter;
use std::fmt::Debug;

pub trait NetworkBehaviourMSyncTrait {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
    fn serialize_sync_vars(&mut self, writer: &mut NetworkWriter, initial_state: bool);
    // DeserializeSyncVars
    fn deserialize_sync_vars(&mut self, reader: &mut NetworkReader, initial_state: bool);
}

// NetworkBehaviourTrait
pub trait NetworkBehaviourTrait: Debug + Send + Sync + NetworkBehaviourMSyncTrait {
    fn sync_var_dirty_bits(&self) -> u64;
}

// Value for NetworkBehaviours type
pub type NetworkBehaviourType = Box<dyn NetworkBehaviourTrait>;

#[derive(Debug)]
pub struct NetworkBehaviour {}

impl NetworkBehaviour {}

#[cfg(test)]
mod tests {}
