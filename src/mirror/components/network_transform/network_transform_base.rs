use crate::commons::revel_weak::RevelWeak;
use crate::metadata_settings::mirror::network_behaviours::metadata_network_transform_base;
use crate::mirror::components::network_transform::transform_snapshot::TransformSnapshot;

use crate::metadata_settings::mirror::network_behaviours::metadata_network_behaviour::MetadataNetworkBehaviourWrapper;
use crate::metadata_settings::mirror::network_behaviours::metadata_network_transform_base::MetadataNetworkTransformBase;
use crate::mirror::network_behaviour::TNetworkBehaviour;
use crate::mirror::NetworkBehaviour;
use crate::unity_engine::Transform;
use crate::unity_engine::{GameObject, MonoBehaviour};
use ordered_float::OrderedFloat;
use std::collections::BTreeMap;
use unity_mirror_macro::{namespace, network_behaviour};

#[derive(Debug, PartialOrd, PartialEq, Clone, Default)]
#[allow(unused)]
pub enum CoordinateSpace {
    #[default]
    Local,
    World,
}

impl Into<CoordinateSpace> for metadata_network_transform_base::CoordinateSpace {
    fn into(self) -> CoordinateSpace {
        match self {
            metadata_network_transform_base::CoordinateSpace::Local => CoordinateSpace::Local,
            metadata_network_transform_base::CoordinateSpace::World => CoordinateSpace::World,
        }
    }
}

#[namespace(prefix = "Mirror")]
#[network_behaviour(parent(NetworkBehaviour), metadata(MetadataNetworkTransformBase))]
pub struct NetworkTransformBase {
    pub target: RevelWeak<Transform>,
    pub server_snapshots: BTreeMap<OrderedFloat<f64>, TransformSnapshot>,
    pub only_sync_on_change: bool,
    pub coordinate_space: CoordinateSpace,
    pub is_client_with_authority: bool,
    pub time_stamp_adjustment: f64,
    pub offset: f64,
    pub sync_position: bool,
    pub sync_rotation: bool,
    pub sync_scale: bool,
    pub compress_rotation: bool,
    pub interpolate_position: bool,
    pub interpolate_rotation: bool,
    pub interpolate_scale: bool,
    pub send_interval_multiplier: u32,
    pub timeline_offset: bool,
    pub buffer_reset_multiplier: u32,
    pub send_interval_counter: u32,
    pub last_send_interval_time: f64,
}

impl MonoBehaviour for NetworkTransformBase {
    fn awake(&mut self) {
        if let Some(parent) = self.parent.get() {
            parent.awake();
        }
        // println!("Mirror: NetworkTransformBase Awake");
    }
    fn start(&mut self) {
        // println!("Mirror: NetworkTransformBase Start");
    }
    fn fixed_update(&mut self) {
        // println!("Mirror: NetworkTransformBase FixedUpdate");
    }
    fn update(&mut self) {
        // println!("Mirror: NetworkTransformBase Update");
    }
    fn late_update(&mut self) {
        // println!("Mirror: NetworkTransformBase LateUpdate");
    }
}

impl NetworkTransformBaseOnChangeCallback for NetworkTransformBase {}

impl TNetworkBehaviour for NetworkTransformBase {
    fn new(
        weak_game_object: RevelWeak<GameObject>,
        metadata: &MetadataNetworkBehaviourWrapper,
    ) -> Self
    where
        Self: Sized,
    {
        let mut base = Self::default();

        {
            let config = metadata.get::<MetadataNetworkTransformBase>();
            // TODO
            config.target.instance_id;
            // base.target
        }

        base
    }
}
