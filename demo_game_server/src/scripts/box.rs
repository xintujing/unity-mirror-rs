use unity_mirror_rs::commons::RevelWeak;
use unity_mirror_rs::macro_namespace::*;
use unity_mirror_rs::macro_network_behaviour::*;
use unity_mirror_rs::metadata_settings::MetadataNetworkBehaviourWrapper;
use unity_mirror_rs::mirror::*;
use unity_mirror_rs::unity_engine::{GameObject, MonoBehaviour, MonoBehaviourAny};


#[namespace(rename = "Box")]
#[network_behaviour(
    parent(NetworkBehaviour),
    metadata(crate::backend_metadata::r#box::MetadataBox)
)]
pub struct BoxScript {}
impl MonoBehaviour for BoxScript {}
impl BoxScriptOnChangeCallback for BoxScript {}
impl TNetworkBehaviour for BoxScript {
    fn new(
        weak_game_object: RevelWeak<GameObject>,
        metadata: &MetadataNetworkBehaviourWrapper,
    ) -> Self
    where
        Self: Sized,
    {
        Self::default()
    }
}