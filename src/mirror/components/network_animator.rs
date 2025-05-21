use crate::commons::revel_arc::RevelArc;
use crate::commons::revel_weak::RevelWeak;
use crate::metadata_settings::mirror::network_behaviours::metadata_network_behaviour::MetadataNetworkBehaviourWrapper;
use crate::mirror::network_behaviour_trait::NetworkBehaviourInstance;
use crate::mirror::NetworkBehaviour;
use crate::unity_engine::MonoBehaviour;
use crate::unity_engine::GameObject;
use std::any::TypeId;
use unity_mirror_macro::{namespace, network_behaviour};

#[namespace(prefix = "Mirror")]
#[network_behaviour(parent(NetworkBehaviour))]
pub struct NetworkAnimator {
    #[sync_variable]
    pub animator_speed: f32,
}

impl MonoBehaviour for NetworkAnimator {
    fn awake(&mut self) {
        println!("Mirror: NetworkAnimator Awake");
    }
}

impl NetworkBehaviourInstance for NetworkAnimator {
    fn instance(
        weak_game_object: RevelWeak<GameObject>,
        metadata: &MetadataNetworkBehaviourWrapper,
    ) -> (
        Vec<(RevelArc<Box<dyn MonoBehaviour>>, TypeId)>,
        RevelWeak<NetworkBehaviour>,
        u8,
        u8,
    )
    where
        Self: Sized,
    {
        if let Some(game_object) = weak_game_object.get() {
            println!("{}", game_object.name);
        }

        let animator = Self::default();

        (
            vec![(
                RevelArc::new(Box::new(animator)),
                TypeId::of::<NetworkAnimator>(),
            )],
            RevelWeak::default(),
            0,
            0,
        )
    }
}
