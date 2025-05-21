use crate::metadata_settings::mirror::network_behaviours::metadata_network_behaviour::MetadataNetworkBehaviourWrapper;
use crate::mirror::network_reader::NetworkReader;
use crate::mirror::network_writer::NetworkWriter;
use crate::unity_engine::MonoBehaviour;

pub trait NetworkBehaviour:
    MonoBehaviour + NetworkBehaviourSerializer + NetworkBehaviourDeserializer
{
    fn new(metadata: &MetadataNetworkBehaviourWrapper) -> Self
    where
        Self: Sized;
    fn on_start_server(&mut self) {}
    fn on_stop_server(&mut self) {}
    fn clear_all_dirty_bits(&mut self);
}

pub trait NetworkBehaviourSerializer {
    fn serialize_sync_objects(&mut self, writer: &mut NetworkWriter, initial_state: bool) {}
    fn serialize_objects_all(&mut self, writer: &mut NetworkWriter) {}
    fn serialize_sync_object_delta(&mut self, writer: &mut NetworkWriter) {}
    fn serialize_sync_vars(&mut self, writer: &mut NetworkWriter, initial_state: bool) {}
}

pub trait NetworkBehaviourDeserializer {
    fn deserialize_sync_objects(&mut self, reader: &mut NetworkReader, initial_state: bool) {}
    fn deserialize_objects_all(&mut self, reader: &mut NetworkReader) {}
    fn deserialize_sync_object_delta(&mut self, reader: &mut NetworkReader) {}
    fn deserialize_sync_vars(&mut self, reader: &mut NetworkReader, initial_state: bool) {}
}
