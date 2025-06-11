use crate::commons::revel_weak::RevelWeak;
use crate::metadata_settings::mirror::network_behaviours::metadata_network_animator::{
    MetadataAnimator, MetadataNetworkAnimator, MetadataParameterType,
};
use crate::metadata_settings::mirror::network_behaviours::metadata_network_behaviour::MetadataNetworkBehaviourWrapper;
use crate::mirror::transport::TransportChannel;
use crate::mirror::NetworkReaderPool;
use crate::mirror::NetworkWriter;
use crate::mirror::TNetworkBehaviour;
use crate::mirror::{
    NetworkBehaviour, NetworkBehaviourOnDeserializer, NetworkBehaviourOnSerializer,
};
use crate::mirror::{NetworkBehaviourDeserializer, NetworkBehaviourSerializer, NetworkReader};
use crate::unity_engine::{GameObject, MonoBehaviour};
use unity_mirror_macro_rs::{
    client_rpc, command, namespace, network_behaviour, parent_on_deserialize, parent_on_serialize,
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
    parameters: Vec<AnimatorParameter>,
}

// sync hook
impl NetworkAnimatorOnChangeCallback for NetworkAnimator {}

// 远程过程调用
impl NetworkAnimator {
    // CmdOnAnimationServerMessage(int stateHash, float normalizedTime, int layerId, float weight, byte[] parameters)
    #[command(NetworkAnimator)]
    fn cmd_on_animation_server_message(
        &self,
        state_hash: i32,
        normalized_time: f32,
        layer_id: i32,
        weight: f32,
        parameters: &[u8],
    ) {
        if !self.client_authority {
            return;
        }

        self.rpc_on_animation_client_message(
            state_hash,
            normalized_time,
            layer_id,
            weight,
            parameters,
        );
    }

    // RpcOnAnimationClientMessage(int stateHash, float normalizedTime, int layerId, float weight, byte[] parameters)
    #[client_rpc(include_owner, channel = TransportChannel::Reliable)]
    fn rpc_on_animation_client_message(
        &self,
        state_hash: i32,
        normalized_time: f32,
        layer_id: i32,
        weight: f32,
        parameters: &[u8],
    ) {
        let _ = state_hash;
        let _ = normalized_time;
        let _ = layer_id;
        let _ = weight;
        let _ = parameters;
    }

    // CmdOnAnimationParametersServerMessage(byte[] parameters)
    #[command(NetworkAnimator)]
    fn cmd_on_animation_parameters_server_message(&self, parameters: &[u8]) {
        if !self.client_authority {
            return;
        }

        self.rpc_on_animation_parameters_client_message(parameters);
    }

    // RpcOnAnimationParametersClientMessage(byte[] parameters)
    #[client_rpc(include_owner, channel = TransportChannel::Reliable)]
    fn rpc_on_animation_parameters_client_message(&self, parameters: &[u8]) {
        let _ = parameters;
    }

    // CmdOnAnimationTriggerServerMessage(int hash)
    #[command(NetworkAnimator)]
    fn cmd_on_animation_trigger_server_message(&self, hash: i32) {
        if !self.client_authority {
            return;
        }

        self.rpc_on_animation_trigger_client_message(hash);
    }

    // RpcOnAnimationTriggerClientMessage(int hash)
    #[client_rpc(channel = TransportChannel::Reliable)]
    fn rpc_on_animation_trigger_client_message(&self, hash: i32) {
        let _ = hash;
    }

    // CmdOnAnimationResetTriggerServerMessage(int hash)
    #[command(NetworkAnimator)]
    fn cmd_on_animation_reset_trigger_server_message(&self, hash: i32) {
        if !self.client_authority {
            return;
        }

        self.rpc_on_animation_reset_trigger_client_message(hash);
    }

    // RpcOnAnimationResetTriggerClientMessage(int hash)
    #[client_rpc(channel = TransportChannel::Reliable)]
    fn rpc_on_animation_reset_trigger_client_message(&self, hash: i32) {
        let _ = hash;
    }

    // CmdSetAnimatorSpeed(float newSpeed)
    #[command(NetworkAnimator)]
    fn cmd_set_animator_speed(&mut self, new_speed: f32) {
        self.set_animator_speed(new_speed);
    }
}

// MonoBehaviour
impl MonoBehaviour for NetworkAnimator {
    fn awake(&mut self) {}

    fn update(&mut self) {}
}
// TNetworkBehaviour
impl TNetworkBehaviour for NetworkAnimator {
    fn new(
        _weak_game_object: RevelWeak<GameObject>,
        metadata: &MetadataNetworkBehaviourWrapper,
    ) -> Self
    where
        Self: Sized,
    {
        let mut animator = Self::default();
        {
            let config = metadata.get::<MetadataNetworkAnimator>();
            animator.initialize(config);
            animator.client_authority = config.client_authority;
            animator.set_animator_speed(1.0)
        }
        animator
    }
}

// 序列化、反序列化相关
impl NetworkAnimator {
    fn initialize(&mut self, metadata: &MetadataNetworkAnimator) {
        self.animator = metadata.animator.clone().into();
        self.parameters = self.animator.parameters.clone();
    }
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
                let _int_value = reader.read_blittable::<i32>();
            } else if par.r#type == AnimatorParameterType::Float {
                let _float_value = reader.read_blittable::<f32>();
            } else if par.r#type == AnimatorParameterType::Bool {
                let _bool_value = reader.read_blittable::<bool>();
            }
        }
    }
}
impl NetworkBehaviourOnSerializer for NetworkAnimator {
    #[parent_on_serialize] // 保证 on_serialize 调用链必须连贯，最上层有对 sync_object 脏位的处理
    fn on_serialize(&mut self, writer: &mut NetworkWriter, initial_state: bool) {
        // base.OnSerialize(writer, initialState);  base -> NetworkBehaviour default impl
        {
            self.serialize_sync_objects(writer, initial_state);
            self.serialize_sync_vars(writer, initial_state);
        }

        if initial_state {
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
}
impl NetworkBehaviourOnDeserializer for NetworkAnimator {
    #[parent_on_deserialize] // 保证 on_deserialize 调用链必须连贯，最上层有对 sync_object 脏位的处理
    fn on_deserialize(&mut self, reader: &mut NetworkReader, initial_state: bool) {
        // base.OnDeserialize(reader, initialState);  base -> NetworkBehaviour default impl
        {
            self.deserialize_sync_objects(reader, initial_state);
            self.deserialize_sync_vars(reader, initial_state);
        }

        if initial_state {
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
}
