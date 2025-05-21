use crate::commons::revel_arc::RevelArc;
use crate::commons::revel_weak::RevelWeak;
use crate::metadata_settings::mirror::metadata_network_identity::{
    MetadataNetworkIdentity, MetadataNetworkIdentityWrapper,
};
use crate::metadata_settings::mirror::network_behaviours::metadata_network_behaviour::MetadataNetworkBehaviourWrapper;
use crate::unity_engine::mirror::network_behaviour_factory::NetworkBehaviourFactory;
use crate::unity_engine::mirror::network_behaviour_trait;
use crate::unity_engine::mirror::network_behaviour_trait::{
    NetworkBehaviour, NetworkBehaviourDeserializer, NetworkBehaviourInstance,
    NetworkBehaviourSerializer,
};
use crate::unity_engine::mirror::network_reader::NetworkReader;
use crate::unity_engine::mirror::network_writer::NetworkWriter;
use crate::unity_engine::mono_behaviour::MonoBehaviour;
use crate::unity_engine::mono_behaviour_factory::MonoBehaviourFactory;
use crate::unity_engine::GameObject;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use unity_mirror_macro::namespace;

#[ctor::ctor]
fn static_init() {
    MonoBehaviourFactory::register::<NetworkIdentity>(|weak_game_object, metadata| {
        let wrapper = metadata
            .as_any()
            .downcast_ref::<MetadataNetworkIdentityWrapper>()
            .unwrap();

        // // let wrappers = metadata.list::<MetadataNetworkIdentityWrapper>();
        // if wrappers.len() < 1 {
        //     panic!("NetworkIdentity requires at least one MetadataNetworkIdentity");
        // }

        let identity =
            NetworkIdentity::instance(weak_game_object, wrapper.get::<MetadataNetworkIdentity>());
        let type_id = identity.type_id();
        vec![(Box::new(identity), type_id)]
    });
}

#[namespace(prefix = "Mirror")]
#[derive(Default)]
pub struct NetworkIdentity {
    net_id: u32,
    component_mapping: HashMap<TypeId, Vec<usize>>,
    network_behaviours: Vec<Vec<RevelWeak<Box<dyn network_behaviour_trait::NetworkBehaviour>>>>,
    // network_behaviours: Vec<WeakRwLock<Box<dyn MonoBehaviour>>>,
}

impl MonoBehaviour for NetworkIdentity {
    fn awake(&mut self) {
        println!("Mirror: NetworkIdentity Awake");
    }
    fn update(&mut self) {
        println!("Mirror: NetworkIdentity Update");
    }
    fn on_destroy(&mut self) {
        println!("Mirror: NetworkIdentity Destroyed");
    }
}

impl NetworkBehaviourInstance for NetworkIdentity {
    fn instance(
        weak_game_object: RevelWeak<GameObject>,
        metadata: &MetadataNetworkBehaviourWrapper,
    ) -> (
        Vec<(RevelArc<Box<dyn MonoBehaviour>>, TypeId)>,
        RevelWeak<crate::unity_engine::mirror::NetworkBehaviour>,
        u8,
        u8,
    )
    where
        Self: Sized,
    {
        todo!()
    }
}

impl NetworkBehaviourSerializer for NetworkIdentity {
    fn serialize(&self, writer: &mut NetworkWriter, initial_state: bool) {
        
    }
}

impl NetworkBehaviourDeserializer for NetworkIdentity {
    fn deserialize(&self, reader: &mut NetworkReader, initial_state: bool) {
        
    }
}

impl NetworkBehaviour for NetworkIdentity {}

impl NetworkIdentity {
    fn instance(
        weak_game_object: RevelWeak<GameObject>,
        settings: &MetadataNetworkIdentity,
    ) -> Self {
        let mut identity = Self {
            net_id: 12366,
            ..Default::default()
        };
        if let Some(game_object) = weak_game_object.get() {
            for metadata_network_behaviour_wrapper in settings.network_behaviours.iter() {
                let final_full_name = metadata_network_behaviour_wrapper.get_final_full_name();
                let (network_behaviours, _, _, _) = NetworkBehaviourFactory::create(
                    &final_full_name,
                    weak_game_object.clone(),
                    metadata_network_behaviour_wrapper,
                );

                let index = identity.network_behaviours.len();
                for (_, type_id) in network_behaviours.iter() {
                    if !identity.component_mapping.contains_key(&type_id) {
                        identity.component_mapping.insert(*type_id, vec![index]);
                    } else {
                        if let Some(mapping) = identity.component_mapping.get_mut(&type_id) {
                            mapping.push(index);
                        };
                    }
                }
                game_object.add_component(network_behaviours);

                //
                // // let (mono_behaviour, type_id) = NetworkBehaviourFactory::create(
                // //     &final_full_name,
                // //     weak_game_object.clone(),
                // //     metadata_network_behaviour_wrapper,
                // // );
                // // let arc_network_behaviour = RevelArc::new(mono_behaviour);
                //
                // identity
                //     .network_behaviours
                //     .push(arc_network_behaviour.downgrade());
                // // identity.network_behaviours.push(WeakRwLock::new(&arc_network_behaviour));
                // let index = identity.network_behaviours.len() - 1;
                // if !identity.component_mapping.contains_key(&type_id) {
                //     identity.component_mapping.insert(type_id, vec![index]);
                // } else {
                //     if let Some(mapping) = identity.component_mapping.get_mut(&type_id) {
                //         mapping.push(index);
                //     };
                // }
                //
                // game_object.add_component(vec![(arc_network_behaviour, type_id)]);
            }
        }

        println!("Mirror: NetworkIdentity Instance");

        identity
    }
}

impl NetworkIdentity {
    pub fn net_id(&self) -> u32 {
        self.net_id
    }
    // pub fn get_component<T: NetworkBehaviour>(&self) -> Option<T> {
    //     let type_id = TypeId::of::<T>();
    //     unsafe {
    //         match self.network_behaviours.get(&type_id) {
    //             None => None,
    //             Some(network_behaviour) => {
    //                 let option = network_behaviour.upgrade();
    //                 let rc = option.unwrap();
    //                 let x = rc.get();
    //                 let x1 = x.as_any_mut().downcast_mut::<T>();
    //             },
    //         }
    //     }
    // }
}
