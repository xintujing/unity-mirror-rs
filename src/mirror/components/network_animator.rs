use crate::commons::revel_weak::RevelWeak;
use crate::metadata_settings::mirror::network_behaviours::metadata_network_animator::{
    MetadataAnimator, MetadataNetworkAnimator, MetadataParameterType,
};
use crate::metadata_settings::mirror::network_behaviours::metadata_network_behaviour::MetadataNetworkBehaviourWrapper;
use crate::mirror::network_behaviour::TNetworkBehaviour;
use crate::mirror::network_reader::NetworkReader;
use crate::mirror::network_reader_pool::NetworkReaderPool;
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

    pub(super) last_int_parameters: Vec<i32>,
    pub(super) last_float_parameters: Vec<f32>,
    pub(super) last_bool_parameters: Vec<bool>,
    pub(super) parameters: Vec<AnimatorParameter>,
    // animation_hash: Vec<i32>,
    // transition_hash: Vec<i32>,
    // layer_weight: Vec<f32>,
    // next_send_time: f64,
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

    fn next_dirty_bits(&mut self) -> u64 {
        let mut dirty_bits = 0u64;
        for (i, par) in self.parameters.iter().enumerate() {
            let mut changed = false;
            if par.r#type == AnimatorParameterType::Int {
                NetworkReaderPool::get_with_slice_return(&par.value, |reader| {
                    let new_int_value = reader.read_blittable::<i32>();

                    changed |= self.last_int_parameters[i] != new_int_value;
                    if changed {
                        self.last_int_parameters[i] = new_int_value;
                    }
                });
            } else if par.r#type == AnimatorParameterType::Float {
                NetworkReaderPool::get_with_slice_return(&par.value, |reader| {
                    let new_float_value = reader.read_blittable::<f32>();
                    changed |= (new_float_value - self.last_float_parameters[i]).abs() > 0.001;
                    if changed {
                        self.last_float_parameters[i] = new_float_value;
                    }
                });
            } else if par.r#type == AnimatorParameterType::Bool {
                NetworkReaderPool::get_with_slice_return(&par.value, |reader| {
                    let new_bool_value = reader.read_blittable::<bool>();
                    changed |= self.last_bool_parameters[i] != new_bool_value;
                    if changed {
                        self.last_bool_parameters[i] = new_bool_value;
                    }
                });
            }
            if changed {
                dirty_bits |= 1 << i;
            }
        }
        dirty_bits
    }

    fn write_parameters(&mut self, writer: &mut NetworkWriter, force_all: bool) -> bool {
        let parameter_count = self.parameters.len() as u8;
        writer.write_blittable::<u8>(parameter_count);

        let dirty_bits: u64;
        if force_all {
            dirty_bits = u64::MAX;
        } else {
            dirty_bits = self.next_dirty_bits();
        }
        writer.write_blittable::<u64>(dirty_bits);
        for (i, par) in self.parameters.iter().enumerate() {
            if dirty_bits & (1 << i) == 0 {
                continue;
            }

            if par.r#type == AnimatorParameterType::Int {
                NetworkReaderPool::get_with_slice_return(&par.value, |reader| {
                    let int_value = reader.read_blittable::<i32>();
                    writer.write_blittable(int_value);
                });
            } else if par.r#type == AnimatorParameterType::Float {
                NetworkReaderPool::get_with_slice_return(&par.value, |reader| {
                    let float_value = reader.read_blittable::<f32>();
                    writer.write_blittable(float_value);
                });
            } else if par.r#type == AnimatorParameterType::Bool {
                NetworkReaderPool::get_with_slice_return(&par.value, |reader| {
                    let bool_value = reader.read_blittable::<bool>();
                    writer.write_blittable(bool_value);
                });
            }
        }
        dirty_bits != 0
    }

    fn read_parameters(&mut self, reader: &mut NetworkReader) {
        let parameter_count = reader.read_blittable::<u8>() as usize;

        if parameter_count != self.parameters.len() {
            log::error!("NetworkAnimator: serialized parameter count={} does not match expected parameter count={}. Are you changing animators at runtime?", parameter_count, self.parameters.len());
            return;
        }

        let dirty_bits = reader.read_blittable::<u64>();

        for i in 0..parameter_count {
            if dirty_bits & (1 << i) == 0 {
                continue;
            }
            let par = &self.animator.parameters[i];
            if par.r#type == AnimatorParameterType::Int {
                let int_value = reader.read_blittable::<i32>();
            } else if par.r#type == AnimatorParameterType::Float {
                let float_value = reader.read_blittable::<f32>();
            } else if par.r#type == AnimatorParameterType::Bool {
                let bool_value = reader.read_blittable::<bool>();
            }
        }
    }
}

impl MonoBehaviour for NetworkAnimator {
    fn awake(&mut self) {}

    fn update(&mut self) {}
}

impl TNetworkBehaviour for NetworkAnimator {
    fn new(
        weak_game_object: RevelWeak<GameObject>,
        metadata: &MetadataNetworkBehaviourWrapper,
    ) -> Self
    where
        Self: Sized,
    {
        Self::default()
    }
}

impl NetworkAnimatorOnChangeCallback for NetworkAnimator {}

impl NetworkBehaviourOnSerializer for NetworkAnimator {
    #[parent_on_serialize]
    fn on_serialize(&mut self, writer: &mut NetworkWriter, initial_state: bool) {
        let ani_layers = &self.animator.layers;
        let layer_count = ani_layers.len() as u8;
        writer.write_blittable(layer_count);

        for layer in ani_layers.iter() {
            writer.write_blittable(layer.full_path_hash);
            writer.write_blittable(layer.normalized_time);
            writer.write_blittable(layer.layer_weight);
        }

        self.write_parameters(writer, true);
    }
}
impl NetworkBehaviourOnDeserializer for NetworkAnimator {
    #[parent_on_deserialize]
    fn on_deserialize(&mut self, reader: &mut NetworkReader, initial_state: bool) {
        let ani_layers = reader.read_blittable::<u8>() as usize;
        if ani_layers != self.animator.layers.len() {
            log::error!("Animator layers count mismatch");
            return;
        }
        for _ in 0..ani_layers {
            let _full_path_hash = reader.read_blittable::<i32>();
            let _normalized_time = reader.read_blittable::<f32>();
            let _layer_weight = reader.read_blittable::<f32>();
        }

        self.read_parameters(reader);
    }
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
