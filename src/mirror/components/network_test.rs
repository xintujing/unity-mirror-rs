use crate::metadata_settings::mirror::network_behaviours::metadata_network_animator::MetadataNetworkAnimator;
use crate::metadata_settings::mirror::network_behaviours::metadata_network_behaviour::MetadataNetworkBehaviourWrapper;
use crate::mirror::network_behaviour_trait::NetworkBehaviourT;
use crate::mirror::sync_list::SyncList;
use crate::mirror::NetworkBehaviour;
use crate::unity_engine::MonoBehaviour;
use unity_mirror_macro::{namespace, network_behaviour};

#[namespace(prefix = "Mirror")]
#[network_behaviour(parent(NetworkBehaviour), metadata(MetadataNetworkAnimator))]
pub struct NetworkTest {
    #[sync_variable]
    pub sync_var_01: f32,
    #[sync_object]
    pub sync_obj_01: SyncList<i32>,
}

impl MonoBehaviour for NetworkTest {
    fn awake(&mut self) {
        println!("Mirror: NetworkAnimator Awake");
    }
}

impl NetworkBehaviourT for NetworkTest {
    fn new(metadata: &MetadataNetworkBehaviourWrapper) -> Self
    where
        Self: Sized,
    {
        Self::default()
    }
}

impl NetworkTestOnChangeCallback for NetworkTest {}
