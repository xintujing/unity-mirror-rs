use crate::commons::Object;
use crate::commons::RevelArc;
use crate::metadata_settings::collider::{MetadataCollider, MetadataColliderWrapper};
use crate::unity_engine::mono_behaviour::MonoBehaviour;
use crate::unity_engine::mono_behaviour_factory::MonoBehaviourFactory;
use std::any::Any;
use crate::namespace;

#[ctor::ctor]
fn static_init() {
    MonoBehaviourFactory::register::<Collider>(|weak_game_object, metadata| {
        let wrapper = metadata
            .as_any()
            .downcast_ref::<MetadataColliderWrapper>()
            .unwrap();

        // let wrappers = metadata.list::<MetadataColliderWrapper>();
        // if wrappers.len() < 1 {
        //     panic!("Collider requires at least one MetadataCollider");
        // }
        let collider = Collider::instance(wrapper.get::<MetadataCollider>());
        let type_id = collider.type_id();

        let arc_collider = RevelArc::new(Box::new(collider) as Box<dyn MonoBehaviour>);

        vec![(arc_collider, type_id)]
    });
}

#[namespace(prefix = "UnityEngine")]
pub struct Collider {}

impl MonoBehaviour for Collider {
    fn awake(&mut self) {
        // println!("UnityEngine: Collider Awake");
    }
    fn update(&mut self) {
        // println!("UnityEngine: Collider Update");
    }
    fn on_destroy(&mut self) {
        // println!("UnityEngine: Collider Destroyed");
    }
}

impl Collider {
    fn instance(_settings: &MetadataCollider) -> Self {
        // println!("UnityEngine: Collider Instance");
        Self {}
    }
}
