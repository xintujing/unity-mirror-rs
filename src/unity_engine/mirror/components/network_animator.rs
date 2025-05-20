use crate::commons::revel_arc::RevelArc;
use crate::commons::revel_weak::RevelWeak;
use crate::metadata_settings::mirror::network_behaviours::metadata_network_behaviour::MetadataNetworkBehaviourWrapper;
use crate::unity_engine::mirror::NetworkBehaviour;
use crate::unity_engine::mono_behaviour::MonoBehaviour;
use crate::unity_engine::GameObject;
use std::any::TypeId;
use unity_mirror_macro::{namespace, network_behaviour};

#[namespace(prefix = "Mirror")]
#[network_behaviour(parent(NetworkBehaviour))]
pub struct NetworkAnimator {}

impl MonoBehaviour for NetworkAnimator {
    fn awake(&mut self) {
        println!("Mirror: NetworkAnimator Awake");
    }
}

impl NetworkAnimator {
    pub fn instance(
        weak_game_object: RevelWeak<GameObject>,
        metadata: &MetadataNetworkBehaviourWrapper,
    ) -> Vec<(RevelArc<Box<dyn MonoBehaviour>>, TypeId)> {
        if let Some(game_object) = weak_game_object.get() {
            println!("{}", game_object.name);
        }

        vec![(
            RevelArc::new(Box::new(NetworkAnimator {
                parent: RevelWeak::new(),
            })),
            TypeId::of::<NetworkAnimator>(),
        )]
    }
}
