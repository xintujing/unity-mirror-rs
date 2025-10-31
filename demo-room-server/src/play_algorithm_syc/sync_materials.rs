use crate::metadata_settings::mirror::metadata_network_behaviour::MetadataNetworkBehaviourWrapper;
use crate::mirror::component::{Component, NetworkBehaviour, UnityEngineLifespan};
use crate::mirror::components::c_sync_var::CSyncVar;
use crate::mirror::identity::Identity;
use crate::mirror::pointer::WeakMutex;
use crate::{default_method_parameter_deserializer, default_method_parameter_serializer};
use dda_macro::{init_state, namespace, Component, ComponentState};

#[derive(Debug, Clone, PartialOrd, PartialEq)]
pub struct MaterialPaths {
    front: String,
    back: String,
}
default_method_parameter_serializer!(
    MaterialPaths,
    { |value, writer| writer.write_blittable(value) }
);
default_method_parameter_deserializer!(
    MaterialPaths,
    { |reader| reader.read_blittable() }
);
#[derive(ComponentState)]
pub struct SyncMaterialsState {
    material_paths: CSyncVar<MaterialPaths>,
}
#[derive(Component, Clone)]
#[namespace(prefix = "HotUpdate.PlayAlgorithmSyc")]
pub struct SyncMaterials {
    id: String,
    #[parent]
    network_behaviour: NetworkBehaviour,
}

impl UnityEngineLifespan for SyncMaterials {}

impl Component for SyncMaterials {
    fn new(behaviour_settings: &MetadataNetworkBehaviourWrapper, weak_mutex_identity: WeakMutex<Identity>, index: u8) -> (Self, usize, usize)
    where
        Self: Sized,
    {
        let (behaviour, sync_obj_index, sync_var_index) = NetworkBehaviour::new(behaviour_settings, weak_mutex_identity, index);
        let (id, sync_obj_index, sync_var_index) = init_state!(
            Some(behaviour.id.clone()),
            (sync_obj_index, sync_var_index),
            SyncMaterialsState {
                material_paths: CSyncVar::new(MaterialPaths {
                    front: "".to_string(),
                    back: "".to_string()
                })
            }
        );

        (
            Self {
                id: id.clone().unwrap(),
                network_behaviour: behaviour,
            },
            sync_obj_index,
            sync_var_index,
        )
    }
}