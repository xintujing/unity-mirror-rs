use crate::commons::revel_arc::RevelArc;
use crate::commons::revel_weak::RevelWeak;
use crate::metadata_settings::mirror::network_behaviours::metadata_network_behaviour::MetadataNetworkBehaviourWrapper;
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
    fn serialize(&self);
}

pub trait NetworkBehaviourDeserializer {
    fn deserialize(&self);
}
