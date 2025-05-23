use crate::metadata_settings::mirror::network_behaviours::metadata_network_behaviour::MetadataNetworkBehaviourWrapper;
use crate::metadata_settings::mirror::network_behaviours::metadata_network_transform_unreliable::MetadataNetworkTransformUnreliable;
use crate::mirror::components::network_transform::network_transform_base::NetworkTransformBase;
use crate::mirror::components::network_transform::transform_snapshot::TransformSnapshot;
use crate::mirror::network_behaviour_trait::NetworkBehaviourT;
use crate::mirror::NetworkBehaviour;
use crate::unity_engine::MonoBehaviour;
use crate::unity_engine::Time;
use unity_mirror_macro::{namespace, network_behaviour};

#[ctor::ctor]
fn static_init() {
    // NetworkBehaviourFactory::register::<NetworkTransformUnreliable>(NetworkTransformUnreliable::instance);
}

#[namespace(prefix = "Mirror")]
#[network_behaviour(
    parent(NetworkTransformBase),
    metadata(MetadataNetworkTransformUnreliable)
)]
pub struct NetworkTransformUnreliable {
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
        let elapsed = Time::unscaled_time().elapsed();
        println!(
            "Mirror: NetworkTransformUnreliable FixedUpdate {:?}",
            elapsed
        );
    }
    fn update(&mut self) {
        // if let Some(parent) = self.parent.get() {
        //     parent.update();
        // }
        let elapsed = Time::unscaled_time().elapsed();
        println!("Mirror: NetworkTransformUnreliable Update {:?}", elapsed);

        let game_object = &self.ancestor.get().unwrap().game_object;

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
        let elapsed = Time::unscaled_time().elapsed();

        println!(
            "Mirror: NetworkTransformUnreliable LateUpdate {:?}",
            elapsed
        );
    }
}

impl NetworkTransformUnreliableOnChangeCallback for NetworkTransformUnreliable {}

impl NetworkBehaviourT for NetworkTransformUnreliable {
    fn new(metadata: &MetadataNetworkBehaviourWrapper) -> Self
    where
        Self: Sized,
    {
        Self::default()
    }
}
