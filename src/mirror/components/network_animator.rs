use crate::metadata_settings::mirror::network_behaviours::metadata_network_animator::MetadataNetworkAnimator;
use crate::metadata_settings::mirror::network_behaviours::metadata_network_behaviour::MetadataNetworkBehaviourWrapper;
use crate::mirror::network_behaviour::TNetworkBehaviour;
use crate::mirror::NetworkBehaviour;
use crate::unity_engine::MonoBehaviour;
use unity_mirror_macro::{client_rpc, namespace, network_behaviour, target_rpc};
use crate::mirror::transport::TransportChannel;

#[namespace(prefix = "Mirror")]
#[network_behaviour(parent(NetworkBehaviour), metadata(MetadataNetworkAnimator))]
pub struct NetworkAnimator {
    #[sync_variable]
    pub animator_speed: f32,
}

impl MonoBehaviour for NetworkAnimator {
    fn awake(&mut self) {
    }

    fn update(&mut self) {
    }
}

impl TNetworkBehaviour for NetworkAnimator {
    fn new(metadata: &MetadataNetworkBehaviourWrapper) -> Self
    where
        Self: Sized,
    {
        Self::default()
    }
}

impl NetworkAnimatorOnChangeCallback for NetworkAnimator {}


impl NetworkAnimator {
    #[target_rpc(channel = TransportChannel::Unreliable)]
    pub fn test_target_rpc(&self) {
        println!("NetworkRoomPlayer: test_target_rpc");
    }

    #[client_rpc(include_owner, channel = TransportChannel::Unreliable)]
    pub fn test_client_rpc(&self) {
        println!("NetworkRoomPlayer: test_target_rpc");
    }
}