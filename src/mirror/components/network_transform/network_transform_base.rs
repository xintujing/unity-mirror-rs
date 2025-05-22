use crate::commons::revel_arc::VecRevelArc;
use crate::commons::revel_weak::RevelWeak;
use crate::metadata_settings::mirror::network_behaviours::metadata_network_transform_base;
use crate::mirror::components::network_transform::transform_snapshot::TransformSnapshot;

use crate::metadata_settings::mirror::network_behaviours::metadata_network_behaviour::MetadataNetworkBehaviourWrapper;
use crate::mirror::network_behaviour_trait::NetworkBehaviourT;
use crate::mirror::NetworkBehaviour;
use crate::unity_engine::MonoBehaviour;
use crate::unity_engine::Transform;
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
#[network_behaviour(parent(NetworkBehaviour))]
pub struct NetworkTransformBase {
    // pub parent: RevelWeak<Box<NetworkBehaviour>>,
    target: RevelWeak<Transform>,
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
        println!("Mirror: NetworkTransformBase Awake");
    }
    fn start(&mut self) {
        println!("Mirror: NetworkTransformBase Start");
    }
    fn fixed_update(&mut self) {
        println!("Mirror: NetworkTransformBase FixedUpdate");
    }
    fn update(&mut self) {
        println!("Mirror: NetworkTransformBase Update");
    }
    fn late_update(&mut self) {
        println!("Mirror: NetworkTransformBase LateUpdate");
    }
}

impl NetworkTransformBaseOnChangeCallback for NetworkTransformBase {}

impl NetworkBehaviourT for NetworkTransformBase {
    fn new(metadata: &MetadataNetworkBehaviourWrapper) -> Self
    where
        Self: Sized,
    {
        todo!()
    }

    fn clear_all_dirty_bits(&mut self) {
        todo!()
    }
}

#[ctor::ctor]
fn static_init() {
    // NetworkBehaviourFactory::register::<NetworkTransformBase>(NetworkTransformBase::instance);
}

// impl NetworkTransformBase {
//     pub fn instance(
//         weak_game_object: RevelWeak<GameObject>,
//         metadata: &MetadataNetworkBehaviourWrapper,
//     ) -> (
//         Vec<(RevelArc<Box<dyn MonoBehaviour>>, TypeId)>,
//         RevelWeak<NetworkBehaviour>,
//         u8,
//         u8,
//     )
//     where
//         Self: Sized,
//     {
//         let (mut network_behaviour_chain, _, _, _) =
//             NetworkBehaviour::instance(weak_game_object.clone(), metadata);
//
//         let mut weak_network_behaviour = RevelWeak::default();
//         if let Some((arc_network_behaviour, _)) = network_behaviour_chain.last() {
//             weak_network_behaviour = arc_network_behaviour.downgrade();
//         }
//
//         let config = metadata.get::<MetadataNetworkTransformBase>();
//
//         let weak_transform = weak_game_object
//             .get()
//             .unwrap()
//             .find_transform(&config.target.instance_id);
//
//         let arc_network_transform_base = RevelArc::new(Box::new(NetworkTransformBase {
//             parent: weak_network_behaviour
//                 .downcast::<NetworkBehaviour>()
//                 .unwrap()
//                 .clone(),
//             target: weak_transform.unwrap(), //_or(RevelWeak::default()),
//             server_snapshots: Default::default(),
//             only_sync_on_change: config.only_sync_on_change,
//             coordinate_space: config.coordinate_space.clone().into(),
//             is_client_with_authority: false,
//             time_stamp_adjustment: 0.0,
//             offset: 0.0,
//             sync_position: config.sync_position,
//             sync_rotation: config.compress_rotation,
//             sync_scale: config.sync_scale,
//             compress_rotation: config.compress_rotation,
//             interpolate_position: config.interpolate_position,
//             interpolate_rotation: config.interpolate_rotation,
//             interpolate_scale: config.interpolate_scale,
//             send_interval_multiplier: 1,
//             timeline_offset: config.timeline_offset,
//             buffer_reset_multiplier: 3,
//             send_interval_counter: 0,
//             last_send_interval_time: 0.0,
//         }) as Box<dyn MonoBehaviour>);
//
//         network_behaviour_chain.push((
//             arc_network_transform_base,
//             TypeId::of::<NetworkTransformBase>(),
//         ));
//         (network_behaviour_chain, RevelWeak::default(), 0, 0)
//     }
// }
