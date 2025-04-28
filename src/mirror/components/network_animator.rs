use crate::mirror::component::component_lifespan::ComponentLifespan;
use crate::mirror::component::state::StateInitialize;
use crate::mirror::components::network_behaviour::NetworkBehaviour;
use unity_mirror_rs_macro::{component, state};

use crate::metadata_settings::mirror::network_behaviours::metadata_network_animator::{
    MetadataAnimator, MetadataNetworkAnimator, MetadataParameterType,
};

use crate::mirror::component::component_factory::ComponentFactory;

impl Into<AnimatorParameterType> for MetadataParameterType {
    fn into(self) -> AnimatorParameterType {
        match self {
            MetadataParameterType::Float => AnimatorParameterType::Float,
            MetadataParameterType::Int => AnimatorParameterType::Int,
            MetadataParameterType::Bool => AnimatorParameterType::Bool,
            MetadataParameterType::Trigger => AnimatorParameterType::Trigger,
        }
    }
}

impl Into<Animator> for MetadataAnimator {
    fn into(self) -> Animator {
        let mut animator = Animator {
            layers: Default::default(),
            parameters: Default::default(),
        };

        self.layers.iter().for_each(|layer| {
            animator.layers.push(AnimatorLayer {
                full_path_hash: layer.full_path_hash,
                normalized_time: layer.normalized_time,
                layer_weight: layer.weight,
            })
        });

        self.parameters
            .iter()
            .enumerate()
            .for_each(|(index, parameter)| {
                animator.parameters.push(AnimatorParameter {
                    index,
                    r#type: parameter.r#type.clone().into(),
                    value: parameter.value.clone(),
                })
            });

        animator
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum AnimatorParameterType {
    Float = 1,
    Int = 3,
    Bool = 4,
    Trigger = 9,
}

#[derive(Debug, Copy, Clone)]
pub struct AnimatorLayer {
    pub full_path_hash: i32,
    pub normalized_time: f32,
    pub layer_weight: f32,
}

#[derive(Debug, Clone)]
pub struct AnimatorParameter {
    #[allow(unused)]
    pub index: usize,
    pub r#type: AnimatorParameterType,
    pub value: Vec<u8>,
}

#[derive(Debug, Clone, Default)]
pub struct Animator {
    pub layers: Vec<AnimatorLayer>,
    pub parameters: Vec<AnimatorParameter>,
}

#[state]
pub struct NetworkAnimatorState {
    pub client_authority: bool,
    pub animator: Animator,

    #[sync_variable]
    pub animator_speed: f32,

    pub previous_speed: f32,
    pub last_int_parameters: Vec<i32>,
    pub last_float_parameters: Vec<f32>,
    pub last_bool_parameters: Vec<bool>,
}

////

impl NetworkAnimatorStateOnChangeCallback for NetworkAnimatorState {
    fn on_animator_speed_changed(&mut self, old_value: &f32, new_value: &f32) {
        println!(
            "NetworkAnimatorState::on_animator_speed_changed: old_value: {:?}, new_value: {:?}",
            old_value, new_value
        )
    }
}

impl StateInitialize for NetworkAnimatorState {
    fn initialize(
        &mut self,
        settings: &crate::metadata_settings::mirror::network_behaviours::metadata_network_behaviour::MetadataNetworkBehaviourWrapper,
    ) where
        Self: Sized,
    {
        let settings = settings.get::<MetadataNetworkAnimator>();
        self.animator = settings.animator.clone().into();
        self.client_authority = settings.client_authority;
    }
}

#[component(
    namespace("Mirror"),
    state(NetworkAnimatorState),
    parent(NetworkBehaviour)
)]
pub struct NetworkAnimator;

impl ComponentLifespan for NetworkAnimator {}

////

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metadata_settings::metadata::Metadata;
    use crate::metadata_settings::mirror::metadata_network_identity::{
        MetadataNetworkIdentity, MetadataNetworkIdentityWrapper,
    };
    use crate::mirror::component::component_basic::ComponentBasic;
    use crate::mirror::component::component_type::ComponentType;

    #[test]
    fn test1() {
        let prefab = Metadata::get_prefab("Assets/Prefabs/Tank.prefab").unwrap();
        let metadata_network_identity_wrapper =
            prefab.components.list::<MetadataNetworkIdentityWrapper>()[0];
        let metadata_network_identity =
            metadata_network_identity_wrapper.get::<MetadataNetworkIdentity>();
        let metadata_network_behaviour_wrapper =
            metadata_network_identity.network_behaviours.get(2).unwrap();

        let instance = NetworkAnimator::instance(&metadata_network_behaviour_wrapper);
        let option = instance.as_any().downcast_ref::<NetworkAnimator>().unwrap();
        let id = instance.id();


        instance.state_clear();

        if let Some(state) = NetworkBehaviour::state(&id) {
            println!("before sync_var_dirty_bit: {}", state.sync_var_dirty_bit);
        }

        if let Some(mut state) = NetworkAnimator::state_mut(&id) {
            state.set_animator_speed(2.0);
        }

        if let Some(state) = NetworkBehaviour::state(&id) {
            println!("after sync_var_dirty_bit: {}", state.sync_var_dirty_bit);
        };
    }
}
