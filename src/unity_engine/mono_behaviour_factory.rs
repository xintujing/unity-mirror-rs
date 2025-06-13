use crate::commons::Object;
use crate::commons::RevelArc;
use crate::commons::RevelWeak;
use crate::metadata_settings::Settings;
use crate::unity_engine::mono_behaviour::MonoBehaviour;
use crate::unity_engine::GameObject;
use once_cell::sync::Lazy;
use std::any::TypeId;
use std::cell::RefCell;
use std::collections::HashMap;

static mut MONO_BEHAVIOUR_FACTORIES: Lazy<
    RefCell<
        HashMap<
            String,
            fn(
                weak_game_object: RevelWeak<GameObject>,
                metadata: &Box<dyn Settings>,
            ) -> Vec<(RevelArc<Box<dyn MonoBehaviour>>, TypeId)>,
        >,
    >,
> = Lazy::new(|| RefCell::new(HashMap::new()));

pub struct MonoBehaviourFactory;

impl MonoBehaviourFactory {
    pub fn register<T: Object>(
        factory: fn(
            weak_game_object: RevelWeak<GameObject>,
            metadata: &Box<dyn Settings>,
        ) -> Vec<(RevelArc<Box<dyn MonoBehaviour>>, TypeId)>,
    ) {
        // log::info!(
        //     "Register MonoBehaviour: {} {}",
        //     std::any::type_name::<T>(),
        //     T::get_full_name()
        // );
        let full_name = T::get_full_name();
        #[allow(static_mut_refs)]
        unsafe {
            if MONO_BEHAVIOUR_FACTORIES.borrow().contains_key(full_name) {
                panic!("Component name {} is already registered", full_name);
            }
            MONO_BEHAVIOUR_FACTORIES
                .borrow_mut()
                .insert(full_name.to_string(), factory);
        }
    }

    pub fn create(
        full_name: &str,
        weak_game_object: RevelWeak<GameObject>,
        settings: &Box<dyn Settings>,
    ) -> Vec<(RevelArc<Box<dyn MonoBehaviour>>, TypeId)> {
        #[allow(static_mut_refs)]
        unsafe {
            match MONO_BEHAVIOUR_FACTORIES.borrow().get(full_name) {
                None => panic!("Component name {} is not registered", full_name),
                Some(factory) => {
                    // let metadata = settings
                    //     .as_any()
                    //     .downcast_ref::<MetadataComponentWrapper>()
                    //     .unwrap();
                    factory(weak_game_object, settings)
                }
            }
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::unity_engine::mono_behaviour::MonoBehaviourMetadata;
//     use crate::unity_engine::transform::Transform;
//     use unity_mirror_rs_macro::namespace;
//
//     #[namespace]
//     struct TestComponent;
//
//     impl MonoBehaviour for TestComponent {
//         fn update(&mut self) {
//             println!("TestComponent update");
//         }
//     }
//
//     #[ctor::ctor]
//     fn init() {
//         MonoBehaviourFactory::register::<TestComponent>(|metadata| Box::new(TestComponent));
//     }
// }
