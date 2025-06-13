use crate::commons::RevelArc;
use crate::commons::RevelWeak;
use crate::metadata_settings::MetadataNetworkManagerWrapper;
use crate::unity_engine::{GameObject, MonoBehaviour};
use std::any::TypeId;

pub trait TNetworkManager: MonoBehaviour + NetworkManagerInstance {}

pub trait NetworkManagerInstance {
    fn instance(
        weak_game_object: RevelWeak<GameObject>,
        metadata: &MetadataNetworkManagerWrapper,
    ) -> Vec<(RevelArc<Box<dyn TNetworkManager>>, TypeId)>
    where
        Self: Sized;
}
