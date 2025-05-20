use crate::commons::revel_arc::RevelArc;
use crate::metadata_settings::mirror::network_behaviours::metadata_network_behaviour::MetadataNetworkBehaviourWrapper;
use crate::unity_engine::mirror::network_behaviour_factory::NetworkBehaviourFactory;
use crate::unity_engine::mono_behaviour::MonoBehaviour;
use crate::unity_engine::GameObject;
use std::any::TypeId;
use unity_mirror_macro::{namespace, network_behaviour};
use crate::unity_engine::mirror::NetworkBehaviour;

#[ctor::ctor]
fn static_init() {
    // NetworkBehaviourFactory::register::<NetworkAnimator>(NetworkAnimator::instance);
}

#[namespace("Mirror")]
#[network_behaviour(parent(NetworkBehaviour))]
pub struct NetworkAnimator {
}

impl MonoBehaviour for NetworkAnimator {
    fn awake(&mut self) {
        println!("Mirror: NetworkAnimator Awake");
    }
}

impl NetworkAnimator {
    // pub fn instance(
    //     weak_game_object: RevelWeak<GameObject>,
    //     metadata: &MetadataNetworkBehaviourWrapper,
    // ) -> Vec<(RevelArc<Box<dyn MonoBehaviour>>, TypeId)> {
    //     if let Some(game_object) = weak_game_object.get() {
    //         println!("{}", game_object.name);
    //     }
    // 
    //     vec![(
    //         RevelArc::new(Box::new(NetworkAnimator {})),
    //         TypeId::of::<NetworkAnimator>(),
    //     )]
    // }
}
