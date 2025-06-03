use crate::metadata_settings::mirror::network_behaviours::metadata_network_behaviour::MetadataNetworkBehaviourWrapper;
use crate::mirror::network_writer::NetworkWriter;

#[allow(unused)]
pub trait State: StateInitialize {
    fn on_serialize_sync_variable(
        &mut self,
        // index: u8,
        dirty_bit: u64,
        writer: &mut NetworkWriter,
        initial: bool,
    ) {
    }

    fn on_serialize_sync_object(
        &mut self,
        // index: u8,
        dirty_bit: u64,
        writer: &mut NetworkWriter,
        initial: bool,
    ) {
    }

    fn on_deserialize_sync_variable(
        &mut self,
        // index: u8,
        // dirty_bit: u64,
        reader: &mut crate::mirror::network_reader::NetworkReader,
        initial: bool,
    ) {
    }

    fn on_deserialize_sync_object(
        &mut self,
        // index: u8,
        dirty_bit: u64,
        reader: &mut crate::mirror::network_reader::NetworkReader,
        initial: bool,
    ) {
    }
}

pub trait StateInitialize {
    #[allow(unused)]
    fn initialize(&mut self, settings: &MetadataNetworkBehaviourWrapper)
    where
        Self: Sized;
}
