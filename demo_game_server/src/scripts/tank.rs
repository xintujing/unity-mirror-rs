use crate::backend_metadata::tank::MetadataTank;
use nalgebra::{Quaternion, Vector3};
use std::any::{Any, TypeId};
use unity_mirror_rs::macro_namespace::*;
use unity_mirror_rs::macro_network_behaviour::*;
use unity_mirror_rs::metadata_settings::{Metadata, MetadataNetworkBehaviourWrapper};
use unity_mirror_rs::mirror::{NetworkConnectionToClient, NetworkServer, SyncList, TNetworkBehaviour};
use unity_mirror_rs::unity_engine::{GameObject, MonoBehaviour, Transform};

#[namespace]
#[network_behaviour(
    parent(NetworkBehaviour),
    metadata(MetadataTank)
)]
pub struct Tank {
    turret: Transform,
    projectile_prefab: String,
    projectile_mount: Transform,
    #[sync_var]
    health: i32,
    #[sync_obj]
    u32_list: SyncList<u32>,
}

impl TankOnChangeCallback for Tank {}

impl MonoBehaviour for Tank {}

impl TNetworkBehaviour for Tank {
    fn new(_weak_game_object: RevelWeak<GameObject>, metadata: &MetadataNetworkBehaviourWrapper) -> Self
    where
        Self: Sized,
    {
        let mut tank = Self::default();
        {
            let config = metadata.get::<MetadataTank>();
            tank.set_health(config.health);
            tank.projectile_prefab = config.projectile_prefab.asset_path.clone();
        }
        tank
    }
}

impl Tank {
    #[command(Tank)]
    fn cmd_fire(&mut self, _pos: Vec<f32>, _rot: Vec<f32>) {
        if let Some(prefab) = Metadata::get_prefab("Assets/Prefabs/Projectile.prefab") {
            let mut obj = GameObject::instantiate(&prefab);
            obj.transform.local_position = Vector3::new(_pos[0], _pos[1], _pos[2]);
            obj.transform.local_rotation = Quaternion::new(_rot[3], _rot[0], _rot[1], _rot[2]);
            NetworkServer::spawn(obj.downgrade());
        }

        self.u32_list.add(1);

        self.rpc_on_fire();
    }

    #[client_rpc]
    fn rpc_on_fire(&self) {}

    #[target_rpc]
    fn target_rpc1(&self) {}
    #[target_rpc]
    fn target_rpc2(&self, conn: RevelArc<Box<NetworkConnectionToClient>>) {}
}
