use crate::commons::revel_arc::RevelArc;
use crate::commons::revel_weak::RevelWeak;
use crate::metadata_settings::mirror::metadata_network_manager::MetadataNetworkManagerWrapper;
use crate::unity_engine::{GameObject, MonoBehaviour};
use std::any::TypeId;

pub trait NetworkManager: MonoBehaviour + NetworkManagerInstance {}

pub trait NetworkManagerInstance {
    fn instance(
        weak_game_object: RevelWeak<GameObject>,
        metadata: &MetadataNetworkManagerWrapper,
    ) -> Vec<(RevelArc<Box<dyn NetworkManager>>, TypeId)>
    where
        Self: Sized;
}
