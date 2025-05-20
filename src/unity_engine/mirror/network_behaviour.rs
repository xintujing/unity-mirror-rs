use crate::commons::revel_arc::RevelArc;
use crate::commons::revel_weak::RevelWeak;
use crate::metadata_settings::mirror::network_behaviours::metadata_network_behaviour::{
    MetadataNetworkBehaviour, MetadataNetworkBehaviourWrapper, MetadataSyncDirection,
    MetadataSyncMode,
};
use crate::unity_engine::mirror::network_behaviour_factory::NetworkBehaviourFactory;
use crate::unity_engine::mirror::NetworkIdentity;
use crate::unity_engine::transform::Transform;
use crate::unity_engine::{GameObject, MonoBehaviour};
use std::any::TypeId;
use unity_mirror_macro::namespace;

#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub enum SyncDirection {
    #[default]
    ServerToClient,
    ClientToServer,
}

impl Into<SyncDirection> for MetadataSyncDirection {
    fn into(self) -> SyncDirection {
        match &self {
            MetadataSyncDirection::ServerToClient => SyncDirection::ServerToClient,
            MetadataSyncDirection::ClientToServer => SyncDirection::ClientToServer,
        }
    }
}

#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub enum SyncMode {
    #[default]
    Observers,
    Owner,
}

impl Into<SyncMode> for MetadataSyncMode {
    fn into(self) -> SyncMode {
        match &self {
            MetadataSyncMode::Observers => SyncMode::Observers,
            MetadataSyncMode::Owner => SyncMode::Owner,
        }
    }
}
// #[network_behaviour(namespace("Mirror"))]
#[namespace("Mirror")]
pub struct NetworkBehaviour {
    sync_direction: SyncDirection,
    sync_mode: SyncMode,
    sync_interval: f32,
    last_sync_time: f64,

    net_id: u32,
    component_index: u8,

    network_identity: RevelWeak<NetworkIdentity>,
    pub game_object: RevelWeak<GameObject>,
    transform: RevelWeak<Transform>,

    sync_var_dirty_bits: u64,
    sync_object_dirty_bits: u64,
}

impl MonoBehaviour for NetworkBehaviour {
    fn awake(&mut self) {
        println!("NetworkBehaviour: awake");
    }
}

#[ctor::ctor]
fn static_init() {
    NetworkBehaviourFactory::register::<NetworkBehaviour>(NetworkBehaviour::instance);
}

impl NetworkBehaviour {
    pub fn instance(
        weak_game_object: RevelWeak<GameObject>,
        metadata: &MetadataNetworkBehaviourWrapper,
    ) -> Vec<(RevelArc<Box<dyn MonoBehaviour>>, TypeId)> {
        let config = metadata.get::<MetadataNetworkBehaviour>();

        let arc_network_behaviour = RevelArc::new(Box::new(NetworkBehaviour {
            sync_direction: config.sync_direction.clone().into(),
            sync_mode: config.sync_mode.clone().into(),
            sync_interval: config.sync_interval,
            last_sync_time: 0.0,
            net_id: 0,
            component_index: 0,
            network_identity: RevelWeak::default(),
            game_object: weak_game_object.clone(),
            transform: weak_game_object.get().unwrap().transform.downgrade(),
            sync_var_dirty_bits: 0,
            sync_object_dirty_bits: 0,
        }) as Box<dyn MonoBehaviour>);

        vec![(arc_network_behaviour, TypeId::of::<NetworkBehaviour>())]
    }
}
