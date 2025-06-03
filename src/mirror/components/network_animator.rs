use crate::commons::revel_weak::RevelWeak;
use crate::metadata_settings::mirror::network_behaviours::metadata_network_animator::{
    MetadataAnimator, MetadataNetworkAnimator, MetadataParameterType,
};
use crate::metadata_settings::mirror::network_behaviours::metadata_network_behaviour::MetadataNetworkBehaviourWrapper;
use crate::mirror::network_behaviour::TNetworkBehaviour;
use crate::mirror::network_reader::NetworkReader;
use crate::mirror::network_writer::NetworkWriter;
use crate::mirror::transport::TransportChannel;
use crate::mirror::{
    NetworkBehaviour, NetworkBehaviourOnDeserializer, NetworkBehaviourOnSerializer,
};
use crate::unity_engine::{GameObject, MonoBehaviour};
use std::ops::{Deref, DerefMut};
use unity_mirror_macro::{
    client_rpc, namespace, network_behaviour, parent_on_deserialize, parent_on_serialize,
    target_rpc,
};

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

#[namespace(prefix = "Mirror")]
#[network_behaviour(
    parent(NetworkBehaviour),
    metadata(MetadataNetworkAnimator),
    not_impl_nos
)]
pub struct NetworkAnimator {
    pub client_authority: bool,
    pub animator: Animator,
    #[sync_variable]
    animator_speed: f32,
    previous_speed: f32,

    last_int_parameters: Vec<i32>,
    last_float_parameters: Vec<f32>,
    last_bool_parameters: Vec<bool>,
    pub(crate) parameters: Vec<AnimatorParameter>,

    animation_hash: Vec<i32>,
    transition_hash: Vec<i32>,
    layer_weight: Vec<f32>,
    next_send_time: f64,
}

impl NetworkAnimator {
    // pub fn send_messages_allowed(&self) -> bool {
    //     if self.is_server() {
    //         if !self.client_authority {
    //             return false;
    //         }
    //     }
    //     self.is_owned() && self.client_authority
    // }

    fn write_parameters(&mut self, writer: &mut NetworkWriter, force_all: bool) {
        let parameter_count = self.parameters.len() as u8;
        writer.write_blittable::<u8>(parameter_count);
    }
}

impl MonoBehaviour for NetworkAnimator {
    fn awake(&mut self) {}

    fn update(&mut self) {}
}

impl TNetworkBehaviour for NetworkAnimator {
    fn new(weak_game_object: &RevelWeak<GameObject>,metadata: &MetadataNetworkBehaviourWrapper) -> Self
    where
        Self: Sized,
    {
        Self::default()
    }
}

impl NetworkAnimatorOnChangeCallback for NetworkAnimator {}

impl NetworkBehaviourOnSerializer for NetworkAnimator {
    #[parent_on_serialize]
    fn on_serialize(&mut self, writer: &mut NetworkWriter, initial_state: bool) {}
}
impl NetworkBehaviourOnDeserializer for NetworkAnimator {
    #[parent_on_deserialize]
    fn on_deserialize(&mut self, reader: &mut NetworkReader, initial_state: bool) {}
}

impl NetworkAnimator {
    #[target_rpc(channel = TransportChannel::Unreliable)]
    pub fn test_target_rpc(&self) {
        println!("NetworkRoomPlayer: test_target_rpc");
    }

    #[client_rpc(include_owner, channel = TransportChannel::Unreliable)]
    pub fn test_client_rpc(&self) {
        println!("NetworkRoomPlayer: test_target_rpc");
    }
}
