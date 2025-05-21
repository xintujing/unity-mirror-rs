use crate::commons::revel_arc::RevelArc;
use crate::commons::revel_weak::RevelWeak;
use crate::metadata_settings::mirror::network_behaviours::metadata_network_behaviour::MetadataNetworkBehaviourWrapper;
use crate::metadata_settings::mirror::network_behaviours::metadata_network_transform_unreliable::MetadataNetworkTransformUnreliable;
use crate::unity_engine::mirror::components::network_transform::network_transform_base::NetworkTransformBase;
use crate::unity_engine::mirror::components::network_transform::transform_snapshot::TransformSnapshot;
use crate::unity_engine::mirror::network_behaviour_factory::NetworkBehaviourFactory;
use crate::unity_engine::mono_behaviour::MonoBehaviour;
use crate::unity_engine::GameObject;
use std::any::TypeId;
use unity_mirror_macro::{namespace, network_behaviour};

#[ctor::ctor]
fn static_init() {
    NetworkBehaviourFactory::register::<NetworkTransformUnreliable>(
        NetworkTransformUnreliable::instance,
    );
}

#[namespace("Mirror")]
// #[network_behaviour(parent(NetworkTransformBase))]
pub struct NetworkTransformUnreliable {
    pub parent: RevelWeak<Box<NetworkTransformBase>>,

    pub buffer_reset_multiplier: f32,
    pub position_sensitivity: f32,
    pub rotation_sensitivity: f32,
    pub scale_sensitivity: f32,
    pub send_interval_counter: u32,
    pub last_send_interval_time: f64,

    pub last_snapshot: TransformSnapshot,
    pub cached_snapshot_comparison: bool,
    pub cached_changed_comparison: u8,
    pub has_sent_unchanged_position: bool,
}

impl MonoBehaviour for NetworkTransformUnreliable {
    fn awake(&mut self) {
        if let Some(parent) = self.parent.get() {
            parent.awake();
        }
        println!("Mirror: NetworkTransformUnreliable Awake");
    }
    fn start(&mut self) {
        println!("Mirror: NetworkTransformUnreliable Start");
    }
    fn fixed_update(&mut self) {
        println!("Mirror: NetworkTransformUnreliable FixedUpdate");
    }
    fn update(&mut self) {
        // if let Some(parent) = self.parent.get() {
        //     parent.update();
        // }
        println!("Mirror: NetworkTransformUnreliable Update");

        let game_object = &self.parent.get().unwrap().parent.get().unwrap().game_object;

        // let game_object = root_game_object.get().unwrap();
        let weak_game_object = game_object
            .get()
            .unwrap()
            .try_get_component::<NetworkTransformUnreliable>()
            .unwrap();

        let x = weak_game_object
            .downcast::<NetworkTransformUnreliable>()
            .unwrap();

        let x1 = x.get().unwrap();

        // let weak_network_transform_unreliable =
        //     weak_game_object.to::<NetworkTransformUnreliable>();
        // let x = weak_network_transform_unreliable.get().unwrap();
        println!("{}", x1.buffer_reset_multiplier);
    }
    fn late_update(&mut self) {
        println!("Mirror: NetworkTransformUnreliable LateUpdate");
    }
}

impl NetworkTransformUnreliable {
    pub fn instance(
        weak_game_object: RevelWeak<GameObject>,
        metadata: &MetadataNetworkBehaviourWrapper,
    ) -> Vec<(RevelArc<Box<dyn MonoBehaviour>>, TypeId)> {
        let mut network_behaviour_chain =
            NetworkTransformBase::instance(weak_game_object, metadata);

        let mut weak_network_transform_base = RevelWeak::default();
        if let Some((arc_network_behaviour, _)) = network_behaviour_chain.last() {
            weak_network_transform_base = arc_network_behaviour.downgrade();
        }
        let config = metadata.get::<MetadataNetworkTransformUnreliable>();

        let arc_mono_behaviour = RevelArc::new(Box::new(NetworkTransformUnreliable {
            parent: weak_network_transform_base
                .downcast::<NetworkTransformBase>()
                .unwrap()
                .clone(),
            buffer_reset_multiplier: config.buffer_reset_multiplier,
            position_sensitivity: config.position_sensitivity,
            rotation_sensitivity: config.rotation_sensitivity,
            scale_sensitivity: config.scale_sensitivity,
            send_interval_counter: 0,
            last_send_interval_time: 0.0,
            last_snapshot: Default::default(),
            cached_snapshot_comparison: false,
            cached_changed_comparison: 0,
            has_sent_unchanged_position: false,
        }) as Box<dyn MonoBehaviour>);

        network_behaviour_chain.push((
            arc_mono_behaviour,
            TypeId::of::<NetworkTransformUnreliable>(),
        ));

        network_behaviour_chain
    }
}
