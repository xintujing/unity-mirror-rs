use crate::metadata_settings::mirror::network_behaviours::metadata_network_behaviour::MetadataNetworkBehaviourWrapper;
use crate::mirror::network_behaviour_trait::NetworkBehaviourT;
use crate::mirror::NetworkBehaviour;
use crate::unity_engine::MonoBehaviour;
use unity_mirror_macro::{namespace, network_behaviour};

#[namespace(prefix = "Mirror")]
#[network_behaviour(parent(NetworkBehaviour))]
pub struct NetworkAnimator {
    #[sync_variable]
    pub animator_speed: f32,
}

impl MonoBehaviour for NetworkAnimator {
    fn awake(&mut self) {
        println!("Mirror: NetworkAnimator Awake");
    }
}

impl NetworkBehaviourT for NetworkAnimator {
    fn new(metadata: &MetadataNetworkBehaviourWrapper) -> Self
    where
        Self: Sized,
    {
        Self::default()
    }

    fn clear_all_dirty_bits(&mut self) {
        todo!()
    }
}

impl NetworkAnimatorOnChangeCallback for NetworkAnimator {}
