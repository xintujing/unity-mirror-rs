use crate::commons::revel_arc::RevelArc;
use crate::commons::revel_weak::RevelWeak;
use crate::metadata_settings::mirror::metadata_network_identity::{
    MetadataNetworkIdentity, MetadataNetworkIdentityWrapper,
};
use crate::metadata_settings::mirror::network_behaviours::metadata_network_behaviour::MetadataNetworkBehaviourWrapper;
use crate::mirror::network_behaviour_factory::NetworkBehaviourFactory;
use crate::mirror::network_behaviour_trait;
use crate::mirror::network_behaviour_trait::{
    NetworkBehaviourDeserializer, NetworkBehaviourSerializer, NetworkBehaviourT,
};
use crate::mirror::network_reader::NetworkReader;
use crate::mirror::network_writer::{DataTypeSerializer, NetworkWriter};
use crate::mirror::network_writer_pool::NetworkWriterPool;
use crate::unity_engine::GameObject;
use crate::unity_engine::MonoBehaviour;
use crate::unity_engine::MonoBehaviourFactory;
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
    network_behaviours: Vec<Vec<RevelWeak<Box<dyn NetworkBehaviourT>>>>,
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

impl NetworkIdentity {
    // ServerDirtyMasks
    fn server_dirty_masks(&self, initial_state: bool) -> (u64, u64) {
        let mut owner_mask = 0u64;
        let mut observer_mask = 0u64;

        // for (i, component) in self.network_behaviours.iter().enumerate() {
        //     let nth_bit = 1u64 << (i as u8);
        //
        //     let dirty = component.is_dirty();
        //
        //     if initial_state || (dirty && (component.get_sync_direction() == SyncDirection::ServerToClient)) {
        //         owner_mask |= nth_bit;
        //     }
        //
        //     if (component.get_sync_mod() == SyncMode::Observers) && (initial_state || dirty) {
        //         observer_mask |= nth_bit;
        //     }
        // }
        (owner_mask, observer_mask)
    }

    fn is_dirty(&self, mask: u64, index: u8) -> bool {
        (mask & (1u64 << index)) != 0
    }

    pub(crate) fn serialize_server(
        &self,
        initial_state: bool,
        owner_writer: &mut NetworkWriter,
        observers_writer: &mut NetworkWriter,
    ) {
        let (owner_mask, observer_mask) = self.server_dirty_masks(initial_state);

        if owner_mask != 0 {
            owner_writer.write_blittable_compress(observer_mask);
        }
        if observer_mask != 0 {
            observers_writer.write_blittable_compress(owner_mask);
        }

        if (owner_mask | observer_mask) != 0 {
            for (network_behaviour_i, network_behaviour) in
                self.network_behaviours.iter().enumerate()
            {
                let owner_dirty = self.is_dirty(owner_mask, network_behaviour_i as u8);
                let observers_dirty = self.is_dirty(observer_mask, network_behaviour_i as u8);

                if owner_dirty || observers_dirty {
                    NetworkWriterPool::get_return(|writer| {
                        // serialize obj
                        for item in network_behaviour.iter() {
                            if let Some(network_behaviour) = item.get() {
                                network_behaviour.serialize_sync_objects(writer, initial_state);
                            }
                        }

                        // serialize var
                        for item in network_behaviour.iter() {
                            if let Some(network_behaviour) = item.get() {
                                network_behaviour.serialize_sync_vars(writer, initial_state);
                            }
                        }

                        // on_serialize
                        for item in network_behaviour.iter() {
                            if let Some(network_behaviour) = item.get() {
                                network_behaviour.on_serialize(writer, initial_state);
                            }
                        }

                        if owner_dirty {
                            owner_writer.write_bytes(writer.to_array(), 0, writer.position);
                        }
                        if observers_dirty {
                            observers_writer.write_bytes(writer.to_array(), 0, writer.position);
                        }
                    });

                    if !initial_state {
                        for item in network_behaviour.iter() {
                            if let Some(network_behaviour) = item.get() {
                                network_behaviour.clear_all_dirty_bits();
                            }
                        }
                    }
                }
            }
        }
    }

    pub(crate) fn deserialize_server(
        &self,
        initial_state: bool,
        owner_reader: &mut NetworkReader,
        observers_reader: &mut NetworkReader,
    ) {
    }
}

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
                let network_behaviours = NetworkBehaviourFactory::create(
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
