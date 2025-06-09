use std::any::{Any, TypeId};
use unity_mirror_macro_rs::{client_rpc, command, namespace, network_behaviour, target_rpc};
use unity_mirror_rs::commons::revel_arc::RevelArc;
use unity_mirror_rs::commons::revel_weak::RevelWeak;
use unity_mirror_rs::metadata_settings::mirror::network_behaviours::metadata_network_behaviour::MetadataNetworkBehaviourWrapper;
use unity_mirror_rs::mirror::sync_list::SyncList;
use unity_mirror_rs::mirror::{NetworkConnectionToClient, TNetworkBehaviour};
use unity_mirror_rs::unity_engine::Transform;
use unity_mirror_rs::unity_engine::{GameObject, MonoBehaviour, MonoBehaviourAny};
#[namespace]
#[network_behaviour(
    parent(unity_mirror_rs::mirror::NetworkBehaviour),
    metadata(crate::backend_metadata::tank::MetadataTank)
)]
pub struct Tank {
    turret: Transform,
    projectile_prefab: String,
    projectile_mount: Transform,
    #[sync_variable]
    health: i32,
    u32_list: SyncList<u32>,
}

impl TankOnChangeCallback for Tank {}

impl MonoBehaviour for Tank {}

impl TNetworkBehaviour for Tank {
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

impl Tank {
    #[command(Tank)]
    fn cmd_fire(&self, _pos: Vec<f32>, _rot: Vec<f32>) {
        self.rpc_on_fire();
    }

    #[client_rpc]
    fn rpc_on_fire(&self) {}

    // #[target_rpc]
    // fn target_rpc1(&self) {}
    // #[target_rpc]
    // fn target_rpc2(&self, conn: RevelArc<Box<unity_mirror_rs::mirror::NetworkConnectionToClient>>) {
    // }
}
