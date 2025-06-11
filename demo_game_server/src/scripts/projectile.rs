use unity_mirror_macro_rs::{namespace, network_behaviour};
use unity_mirror_rs::commons::revel_weak::RevelWeak;
use unity_mirror_rs::metadata_settings::mirror::network_behaviours::metadata_network_behaviour::MetadataNetworkBehaviourWrapper;
use unity_mirror_rs::mirror::{NetworkServer, NetworkTime, TNetworkBehaviour};
use unity_mirror_rs::unity_engine::{GameObject, MonoBehaviour};


#[namespace]
#[network_behaviour(
    parent(unity_mirror_rs::mirror::NetworkBehaviour),
    metadata(crate::backend_metadata::projectile::MetadataProjectile)
)]
pub struct Projectile {
    z: f64,
}

impl ProjectileOnChangeCallback for Projectile {}
impl TNetworkBehaviour for Projectile {
    fn new(weak_game_object: RevelWeak<GameObject>, metadata: &MetadataNetworkBehaviourWrapper) -> Self
    where
        Self: Sized,
    {
        let projectile = Self::default();
        projectile
    }
    fn on_start_server(&mut self) {
        self.z = NetworkTime.local_time();
    }
}

impl MonoBehaviour for Projectile {
    fn update(&mut self) {
        if NetworkTime.local_time() - self.z >= 2f64 {
            NetworkServer::destroy(self.game_object.clone())
        }
    }
}
