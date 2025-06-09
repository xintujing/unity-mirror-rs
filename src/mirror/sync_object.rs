use crate::commons::revel_weak::RevelWeak;
use crate::mirror::NetworkReader;
use crate::mirror::NetworkWriter;
use crate::mirror::{NetworkBehaviour};
use std::fmt::Debug;
use crate::commons::revel_arc::RevelArc;

#[allow(unused)]
pub trait SyncObject: Default + Debug {
    type Item;
    fn new() -> Self;
    fn new_with_value(value: Self::Item) -> Self;
    fn is_recording(&self) -> bool {
        true
    }
    fn is_writable(&self) -> bool {
        true
    }
    fn set_network_behaviour(&mut self, network_behaviour: RevelWeak<Box<NetworkBehaviour>>);
    fn network_behaviour(&self) -> &RevelWeak<Box<NetworkBehaviour>>;
    fn set_index(&mut self, index: u8);
    fn index(&self) -> u8;
    fn clear_changes(&mut self);
    fn on_serialize_all(&self, writer: &mut NetworkWriter);
    fn on_serialize_delta(&self, writer: &mut NetworkWriter);
    fn on_deserialize_all(&mut self, reader: &mut NetworkReader);
    fn on_deserialize_delta(&mut self, reader: &mut NetworkReader);
    fn reset(&mut self);
    fn on_dirty(&self) {
        // 更新 dirty_bits
        if let Some(network_behaviour) = self.network_behaviour().get() {
            network_behaviour.sync_object_dirty_bits =
                network_behaviour.sync_object_dirty_bits | (1 << self.index());
        } else {
            log::error!("[on_dirty] Failed to get NetworkBehaviour");
        }
    }
}
