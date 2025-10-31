use crate::metadata_settings::mirror::metadata_network_behaviour::MetadataNetworkBehaviourWrapper;
use crate::mirror::component::{Component, NetworkBehaviour, UnityEngineLifespan};
use crate::mirror::components::c_sync_var::CSyncVar;
use crate::mirror::identity::Identity;
use crate::mirror::pointer::WeakMutex;
use dda_macro::{init_state, namespace, Component, ComponentState};

#[derive(ComponentState)]
pub struct SyncVirtualCutCardTrackingCameraState {
    is_cut_card_tracking_camera_active: CSyncVar<bool>,
    cut_card_tracking_camera_priority: CSyncVar<i32>,
}
#[derive(Component, Clone)]
#[namespace(prefix = "HotUpdate.VirtualCameraSys")]
pub struct SyncVirtualCutCardTrackingCamera {
    id: String,
    #[parent]
    network_behaviour: NetworkBehaviour,
}

impl UnityEngineLifespan for SyncVirtualCutCardTrackingCamera {}

impl Component for SyncVirtualCutCardTrackingCamera {
    fn new(behaviour_settings: &MetadataNetworkBehaviourWrapper, weak_mutex_identity: WeakMutex<Identity>, index: u8) -> (Self, usize, usize)
    where
        Self: Sized,
    {
        let (behaviour, sync_obj_index, sync_var_index) = NetworkBehaviour::new(behaviour_settings, weak_mutex_identity, index);
        let (id, sync_obj_index, sync_var_index) = init_state!(
            Some(behaviour.id.clone()),
            (sync_obj_index, sync_var_index),
            SyncVirtualCutCardTrackingCameraState {
                is_cut_card_tracking_camera_active: CSyncVar::new(Default::default()),
                cut_card_tracking_camera_priority:  CSyncVar::new(Default::default()),
            }
        );

        (
            Self {
                id: id.clone().unwrap(),
                network_behaviour: behaviour,
            },
            sync_obj_index,
            sync_var_index,
        )
    }
}