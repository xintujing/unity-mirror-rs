use crate::commons::revel_arc::RevelArc;
use crate::commons::revel_weak::RevelWeak;
use crate::metadata_settings::mirror::network_behaviours::metadata_network_behaviour::MetadataNetworkBehaviourWrapper;
use crate::unity_engine::mono_behaviour::MonoBehaviour;
use crate::unity_engine::GameObject;
use once_cell::sync::Lazy;
use std::any::TypeId;
use std::collections::HashMap;

static mut NETWORK_BEHAVIOUR_FACTORY: Lazy<
    HashMap<
        String,
        fn(
            weak_game_object: RevelWeak<GameObject>,
            metadata: &MetadataNetworkBehaviourWrapper,
        ) -> (
            Vec<(RevelArc<Box<dyn MonoBehaviour>>, TypeId)>,
            RevelWeak<crate::unity_engine::mirror::network_behaviour::NetworkBehaviour>,
            u8,
            u8,
        ),
    >,
> = Lazy::new(|| HashMap::new());

pub struct NetworkBehaviourFactory;
impl NetworkBehaviourFactory {
    pub fn register<T: MonoBehaviour + 'static>(
        factory: fn(
            weak_game_object: RevelWeak<GameObject>,
            metadata: &MetadataNetworkBehaviourWrapper,
        ) -> (
            Vec<(RevelArc<Box<dyn MonoBehaviour>>, TypeId)>,
            RevelWeak<crate::unity_engine::mirror::network_behaviour::NetworkBehaviour>,
            u8,
            u8,
        ),
    ) {
        let full_name = T::get_full_name();
        #[allow(static_mut_refs)]
        unsafe {
            if NETWORK_BEHAVIOUR_FACTORY.contains_key(full_name) {
                panic!(
                    "NetworkBehaviourFactory: Duplicate registration for {}",
                    full_name
                );
            }
            NETWORK_BEHAVIOUR_FACTORY.insert(full_name.to_string(), factory);
        }
    }

    pub fn create(
        full_name: &str,
        weak_game_object: RevelWeak<GameObject>,
        metadata: &MetadataNetworkBehaviourWrapper,
    ) -> (
        Vec<(RevelArc<Box<dyn MonoBehaviour>>, TypeId)>,
        RevelWeak<crate::unity_engine::mirror::network_behaviour::NetworkBehaviour>,
        u8,
        u8,
    ) {
        #[allow(static_mut_refs)]
        unsafe {
            if let Some(factory) = NETWORK_BEHAVIOUR_FACTORY.get(full_name) {
                factory(weak_game_object, metadata)
            } else {
                panic!(
                    "NetworkBehaviourFactory: No factory registered for {}",
                    full_name
                );
            }
        }
    }
}
