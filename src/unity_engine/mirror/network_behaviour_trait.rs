use crate::unity_engine::mono_behaviour::MonoBehaviour;

pub trait NetworkBehaviour:
    MonoBehaviour + NetworkBehaviourSerializer + NetworkBehaviourDeserializer
{
    fn on_start_server(&self) {}
    fn on_stop_server(&self) {}
}

pub trait NetworkBehaviourSerializer {
    fn serialize(&self);
}

pub trait NetworkBehaviourDeserializer {
    fn deserialize(&self);
}
