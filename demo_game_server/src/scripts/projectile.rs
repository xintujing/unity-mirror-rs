// use crate::metadata_settings::mirror::metadata_network_behaviour::MetadataNetworkBehaviourWrapper;
// use crate::mirror::component::{Component, NetworkBehaviour, UnityEngineLifespan};
// use crate::mirror::identity::Identity;
// use crate::mirror::network_server::NetworkServer;
// use crate::mirror::pointer::WeakMutex;
// use crate::mirror::timer::TimerState;
// use crate::tanks_business::backend_metadata::projectile::ProjectileSettings;
// use dda_macro::{Component, ComponentState, init_state, namespace};
use unity_mirror_macro_rs::{namespace, network_behaviour};
use unity_mirror_rs::commons::revel_weak::RevelWeak;
use unity_mirror_rs::metadata_settings::mirror::network_behaviours::metadata_network_behaviour::MetadataNetworkBehaviourWrapper;
use unity_mirror_rs::mirror::{NetworkServer, NetworkTime, TNetworkBehaviour};
use unity_mirror_rs::unity_engine::{GameObject, MonoBehaviour};

// #[derive(ComponentState)]
// pub struct ProjectileState {
//     z: f64,
// }

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

impl MonoBehaviour for Projectile {
    fn update(&mut self) {
        if NetworkTime.local_time() - self.z >= 2f64 {
            NetworkServer::destroy(self.game_object.clone())
        }
    }
}

// impl Component for Projectile {
//     fn new(
//         behaviour_settings: &MetadataNetworkBehaviourWrapper,
//         weak_mutex_identity: WeakMutex<Identity>,
//         index: u8,
//     ) -> (Self, usize, usize)
//     where
//         Self: Sized,
//     {
//         // parent
//         let (behaviour, sync_obj_index, sync_var_index) =
//             NetworkBehaviour::new(behaviour_settings, weak_mutex_identity, index);
//
//         // 获取 NetworkBehaviourSetting
//         let _settings = behaviour_settings.get::<ProjectileSettings>();
//
//         let (id, sync_obj_index, sync_var_index) = init_state!(
//             Some(behaviour.id.clone()),
//             (sync_var_index, sync_obj_index),
//             ProjectileState {
//                 z: TimerState::local_time()
//             }
//         );
//         (
//             Self {
//                 id: id.clone().unwrap(),
//                 network_behaviour: behaviour,
//             },
//             sync_obj_index,
//             sync_var_index,
//         )
//     }
//
//     fn on_start_server(&self) {
//         // let network_behaviour_id = self.network_behaviour.id.clone();
//         // tokio::spawn(async move {
//         //     sleep(Duration::from_secs(2)).await;
//         //     if let Ok(state) = NetworkBehaviour::state(&network_behaviour_id) {
//         //         if let Some(arc_identity) = state.net_identity.clone().upgrade() {
//         //             if let Ok(mut identity) = arc_identity.try_lock() {
//         //                 println!("开始 子弹销毁成功");
//         //
//         //                 NetworkServer::destroy(&mut identity);
//         //                 println!("完成 子弹销毁成功");
//         //             }else {
//         //                 println!("子弹销毁失败，锁定失败 locked");
//         //             }
//         //         }
//         //     }
//         // });
//     }
// }
