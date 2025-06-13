use std::any::Any;
use unity_mirror_rs::commons::Object;
use unity_mirror_rs::commons::RevelWeak;
use unity_mirror_rs::commons::RevelArc;
use unity_mirror_rs::metadata_settings::MetadataNetworkBehaviourWrapper;
use unity_mirror_rs::mirror::{NetworkBehaviour, TNetworkBehaviour};
use unity_mirror_rs::unity_engine::{GameObject, MonoBehaviour, MonoBehaviourAny};
use unity_mirror_rs::{namespace, network_behaviour, SyncState};
use unity_mirror_rs::mirror::*;


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