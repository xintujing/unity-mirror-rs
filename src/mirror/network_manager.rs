use crate::commons::action::SelfMutAction;
use crate::commons::revel_arc::RevelArc;
use crate::commons::revel_weak::RevelWeak;
use crate::metadata_settings::metadata::Metadata;
use crate::metadata_settings::mirror::metadata_network_manager::MetadataNetworkManagerWrapper;
use crate::mirror::authenticator::authenticator::Authenticator;
use crate::mirror::network_manager_factory::NetworkManagerFactory;
use crate::mirror::network_manager_trait;
use crate::unity_engine::{GameObject, MonoBehaviour, WorldManager};
use once_cell::sync::Lazy;
use std::any::Any;
use unity_mirror_macro::{callbacks, namespace, network_manager, NetworkManagerFactory};

#[network_manager]
#[namespace(prefix = "Mirror")]
#[derive(NetworkManagerFactory)]
#[callbacks({
    on_start_server(&mut self);
    on_stop_server(&mut self);
})]
pub struct NetworkManager {
    pub authenticator: Option<Box<dyn Authenticator>>,

    // Action Begin
    pub on_client_scene_changed: SelfMutAction<(), ()>,
    // Action End
}

impl MonoBehaviour for NetworkManager {
    fn awake(&mut self) {
        println!("Mirror: NetworkManager Awake");
    }
    fn update(&mut self) {
        self.on_client_scene_changed.call(());
        // if let Some(ref mut on_client_scene_changed) = self.on_client_scene_changed {
        //     on_client_scene_changed.invoke(());
        // }

        println!("Mirror: NetworkManager Update");
        if let Some(callbacks) = self.callbacks.get() {
            callbacks.on_start_server();
        } else {
            // default code
            println!("Mirror: NetworkManager Default callbacks");
        }
    }
}

impl NetworkManagerInitialize for NetworkManager {
    fn initialize(&mut self, metadata: &MetadataNetworkManagerWrapper) {}
}

static mut NETWORK_MANAGER: Lazy<RevelWeak<Box<dyn network_manager_trait::NetworkManager>>> =
    Lazy::new(|| RevelWeak::default());
impl NetworkManager {
    pub fn start(prefab_path: &str) {
        let metadata = Metadata::get_network_manager(prefab_path).unwrap();
        let full_name = metadata.get_final_full_name();

        let mut arc_game_object = RevelArc::new(GameObject::default());

        let instances =
            NetworkManagerFactory::create(&full_name, arc_game_object.downgrade(), metadata);

        if let Some((instance, last_type_id)) = instances.last() {
            #[allow(static_mut_refs)]
            unsafe {
                *NETWORK_MANAGER = instance.downgrade();
            }
        }
        let instances = instances
            .into_iter()
            .map(|(instance, type_id)| {
                let instance = unsafe {
                    &*(&instance as *const dyn Any as *const RevelArc<Box<dyn MonoBehaviour>>)
                };
                (instance.clone(), type_id)
            })
            .collect::<Vec<_>>();

        arc_game_object.add_component(instances);
        WorldManager::dont_destroy_object(arc_game_object);
    }

    pub fn singleton<T: network_manager_trait::NetworkManager + 'static>(f: fn(&mut T)) {
        #[allow(static_mut_refs)]
        unsafe {
            if let Some(weak) = NETWORK_MANAGER.downcast::<T>() {
                if let Some(real) = weak.get() {
                    f(real)
                }
            }
        }
    }
}
