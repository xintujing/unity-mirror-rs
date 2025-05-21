use crate::commons::revel_arc::RevelArc;
use crate::commons::revel_weak::RevelWeak;
use crate::metadata_settings::mirror::network_behaviours::metadata_network_behaviour::MetadataNetworkBehaviourWrapper;
use crate::unity_engine::mirror::network_reader::NetworkReader;
use crate::unity_engine::mirror::network_writer::NetworkWriter;
use crate::unity_engine::mono_behaviour::MonoBehaviour;
use crate::unity_engine::GameObject;
use std::any::TypeId;

pub trait NetworkBehaviour:
    MonoBehaviour + NetworkBehaviourInstance + NetworkBehaviourSerializer + NetworkBehaviourDeserializer
{
    fn on_start_server(&self) {}
    fn on_stop_server(&self) {}
}

pub trait NetworkBehaviourInstance {
    fn instance(
        weak_game_object: RevelWeak<GameObject>,
        metadata: &MetadataNetworkBehaviourWrapper,
    ) -> (
        Vec<(RevelArc<Box<dyn MonoBehaviour>>, TypeId)>,
        RevelWeak<crate::unity_engine::mirror::network_behaviour::NetworkBehaviour>,
        u8,
        u8,
    )
    where
        Self: Sized;
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
