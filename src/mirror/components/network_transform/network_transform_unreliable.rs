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
    // pub parent: RevelWeak<Box<NetworkTransformBase>>,
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

// impl NetworkTransformUnreliable {
//     fn instance(
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
//             NetworkTransformBase::instance(weak_game_object, metadata);
//
//         let mut weak_network_transform_base = RevelWeak::default();
//         if let Some((arc_network_behaviour, _)) = network_behaviour_chain.last() {
//             if let Some(wnb) = arc_network_behaviour
//                 .downgrade()
//                 .downcast::<NetworkTransformBase>()
//             {
//                 weak_network_transform_base = wnb.clone();
//             }
//         }
//         let config = metadata.get::<MetadataNetworkTransformUnreliable>();
//
//         let arc_mono_behaviour = RevelArc::new(Box::new(NetworkTransformUnreliable {
//             parent: weak_network_transform_base,
//             buffer_reset_multiplier: config.buffer_reset_multiplier,
//             position_sensitivity: config.position_sensitivity,
//             rotation_sensitivity: config.rotation_sensitivity,
//             scale_sensitivity: config.scale_sensitivity,
//             send_interval_counter: 0,
//             last_send_interval_time: 0.0,
//             last_snapshot: Default::default(),
//             cached_snapshot_comparison: false,
//             cached_changed_comparison: 0,
//             has_sent_unchanged_position: false,
//         }) as Box<dyn MonoBehaviour>);
//
//         network_behaviour_chain.push((
//             arc_mono_behaviour,
//             TypeId::of::<NetworkTransformUnreliable>(),
//         ));
//
//         (network_behaviour_chain, RevelWeak::default(), 0, 0)
//     }
// }

// impl NetworkTransformUnreliable {
//     // pub parent: RevelWeak<Box<NetworkTransformBase>>,
//     pub fn factory(weak_game_object: crate::commons::revel_weak::RevelWeak<crate::unity_engine::GameObject>, metadata: &crate::metadata_settings::mirror::network_behaviours::metadata_network_behaviour::MetadataNetworkBehaviourWrapper, weak_network_behaviour: &mut crate::commons::revel_weak::RevelWeak<crate::mirror::network_behaviour::NetworkBehaviour, >, sync_object_offset: &mut u8, sync_var_offset: &mut u8) -> Vec<(crate::commons::revel_arc::RevelArc<Box<dyn crate::unity_engine::MonoBehaviour>>, std::any::TypeId,)> {
//         use super::NetworkBehaviour;
//
//         let mut network_behaviour_chain = #parent::factory(weak_game_object.clone(), metadata, weak_network_behaviour, sync_object_offset, sync_var_offset);
//
//         // 祖先弱指针
//         let mut weak_ancestor = crate::commons::revel_weak::RevelWeak::default();
//         if let Some((arc_nb, _)) = network_behaviour_chain.first() {
//             if let Some(ancestor) = arc_nb.downgrade().downcast::<NetworkBehaviour>() {
//                 weak_ancestor = ancestor.clone();
//             }
//         }
//
//         // 父亲弱指针
//         let mut weak_parent = crate::commons::revel_weak::RevelWeak::default();
//         if let Some((arc_nb, _)) = network_behaviour_chain.last() {
//             if let Some(parent) = arc_nb.downgrade().downcast
//             :: < # parent > ()
//             {
//                 weak_parent = parent.clone();
//             }
//         }
//
//         let config = metadata.get::<#metadata>();
//
//         let this = Self::new(metadata);
//         let arc_this = crate::commons::revel_arc::RevelArc::new(Box::new(this) as Box<dyn crate::unity_engine::MonoBehaviour>);
//
//         network_behaviour_chain.push((
//             arc_this,
//             std::any::TypeId::of::<Self>(),
//         ));
//
//         network_behaviour_chain
//     }
// }
