use crate::commons::revel_arc::RevelArc;
use crate::commons::revel_weak::RevelWeak;
use crate::metadata_settings::mirror::network_behaviours::metadata_network_behaviour::MetadataNetworkBehaviourWrapper;
use crate::unity_engine::GameObject;
use crate::unity_engine::MonoBehaviour;
use once_cell::sync::Lazy;
use std::any::TypeId;
use std::collections::HashMap;

pub type NetworkBehaviourFactoryType = fn(
    weak_game_object: RevelWeak<GameObject>,
    metadata: &MetadataNetworkBehaviourWrapper,
    weak_network_behaviour: &mut RevelWeak<Box<crate::mirror::network_behaviour::NetworkBehaviour>>,
    sync_object_offset: &mut u8,
    sync_var_offset: &mut u8,
) -> Vec<(RevelArc<Box<dyn MonoBehaviour>>, TypeId)>;

static mut NETWORK_BEHAVIOUR_FACTORY: Lazy<HashMap<String, NetworkBehaviourFactoryType>> =
    Lazy::new(|| HashMap::new());

pub struct NetworkBehaviourFactory;
impl NetworkBehaviourFactory {
    pub fn register<T: MonoBehaviour + 'static>(factory: NetworkBehaviourFactoryType) {
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
    ) -> Vec<(RevelArc<Box<dyn MonoBehaviour>>, TypeId)> {
        #[allow(static_mut_refs)]
        unsafe {
            if let Some(factory) = NETWORK_BEHAVIOUR_FACTORY.get(full_name) {
                factory(
                    weak_game_object,
                    metadata,
                    &mut RevelWeak::default(),
                    &mut 0,
                    &mut 0,
                )
            } else {
                panic!(
                    "NetworkBehaviourFactory: No factory registered for {}",
                    full_name
                );
            }
        }
    }
}
