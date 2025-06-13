use crate::commons::Object;
use crate::commons::RevelArc;
use crate::metadata_settings::rigid_body::{MetadataRigidBody, MetadataRigidBodyWrapper};
use crate::unity_engine::mono_behaviour::MonoBehaviour;
use crate::unity_engine::mono_behaviour_factory::MonoBehaviourFactory;
use std::any::Any;
use crate::namespace;

#[ctor::ctor]
fn static_init() {
    MonoBehaviourFactory::register::<RigidBody>(|weak_game_object, metadata| {
        let wrapper = metadata
            .as_any()
            .downcast_ref::<MetadataRigidBodyWrapper>()
            .unwrap();

        // let wrappers = metadata.list::<MetadataRigidBodyWrapper>();
        // if wrappers.len() < 1 {
        //     panic!("RigidBody requires at least one MetadataRigidBody");
        // }
        let rigid_body = RigidBody::instance(wrapper.get::<MetadataRigidBody>());
        let type_id = rigid_body.type_id();

        let arc_rigid_body = RevelArc::new(Box::new(rigid_body) as Box<dyn MonoBehaviour>);

        vec![(arc_rigid_body, type_id)]
    });
}

#[namespace(prefix = "UnityEngine", rename = "Rigidbody")]
pub struct RigidBody {}

impl MonoBehaviour for RigidBody {
    fn awake(&mut self) {
        // println!("UnityEngine: RigidBody Awake");
    }
    fn update(&mut self) {
        // println!("UnityEngine: RigidBody Update");
    }
    fn on_destroy(&mut self) {
        // println!("UnityEngine: RigidBody Destroyed");
    }
}

impl RigidBody {
    fn instance(_settings: &MetadataRigidBody) -> Self {
        Self {}
    }
}
