use crate::commons::RevelArc;
use crate::commons::RevelWeak;
use crate::metadata_settings::MetadataNetworkManagerWrapper;
use crate::mirror::TNetworkManager;
use crate::unity_engine::GameObject;
use once_cell::sync::Lazy;
use std::any::TypeId;
use std::collections::HashMap;

static mut NETWORK_MANAGER_FACTORY: Lazy<
    HashMap<
        String,
        fn(
            weak_game_object: RevelWeak<GameObject>,
            metadata: &MetadataNetworkManagerWrapper,
        ) -> Vec<(RevelArc<Box<dyn TNetworkManager>>, TypeId)>,
    >,
> = Lazy::new(|| HashMap::new());

pub struct NetworkManagerFactory;

impl NetworkManagerFactory {
    pub fn register<T: TNetworkManager + 'static>(
        factory: fn(
            weak_game_object: RevelWeak<GameObject>,
            metadata: &MetadataNetworkManagerWrapper,
        ) -> Vec<(RevelArc<Box<dyn TNetworkManager>>, TypeId)>,
    ) {
        let full_name = T::get_full_name();
        #[allow(static_mut_refs)]
        unsafe {
            if NETWORK_MANAGER_FACTORY.contains_key(full_name) {
                panic!(
                    "NetworkManagerFactory: Duplicate registration for {}",
                    full_name
                );
            }
            NETWORK_MANAGER_FACTORY.insert(full_name.to_string(), factory);
        }
    }

    pub fn create(
        full_name: &str,
        weak_game_object: RevelWeak<GameObject>,
        metadata: &MetadataNetworkManagerWrapper,
    ) -> Vec<(RevelArc<Box<dyn TNetworkManager>>, TypeId)> {
        #[allow(static_mut_refs)]
        unsafe {
            if let Some(factory) = NETWORK_MANAGER_FACTORY.get(full_name) {
                return factory(weak_game_object, metadata);
            }
        }
        panic!(
            "NetworkManagerFactory: No factory registered for {}",
            full_name
        );
    }
}
