use crate::metadata_settings::unity::collider::{MetadataCollider, MetadataColliderWrapper};
use crate::metadata_settings::unity::rigid_body::MetadataRigidBodyWrapper;
use crate::unity_engine::mono_behaviour::MonoBehaviour;
use crate::unity_engine::mono_behaviour_factory::MonoBehaviourFactory;
use std::any::Any;
use unity_mirror_macro::namespace;

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
        vec![(Box::new(collider), type_id)]
    });
}

#[namespace("UnityEngine")]
pub struct Collider {}

impl MonoBehaviour for Collider {
    fn awake(&mut self) {
        println!("UnityEngine: Collider Awake");
    }
    fn update(&mut self) {
        println!("UnityEngine: Collider Update");
    }
    fn on_destroy(&mut self) {
        println!("UnityEngine: Collider Destroyed");
    }
}

impl Collider {
    fn instance(settings: &MetadataCollider) -> Self {
        println!("UnityEngine: Collider Instance");
        Self {}
    }
}
