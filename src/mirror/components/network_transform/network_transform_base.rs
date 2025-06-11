use crate::commons::revel_weak::RevelWeak;
use crate::metadata_settings::mirror::network_behaviours::metadata_network_transform_base;
use crate::mirror::components::network_transform::transform_snapshot::TransformSnapshot;

use crate::metadata_settings::mirror::network_behaviours::metadata_network_behaviour::MetadataNetworkBehaviourWrapper;
use crate::metadata_settings::mirror::network_behaviours::metadata_network_transform_base::MetadataNetworkTransformBase;
use crate::mirror::transport::TransportChannel;
use crate::mirror::TNetworkBehaviour;
use crate::mirror::{NetworkBehaviour, NetworkServer, SyncDirection};
use crate::unity_engine::Transform;
use crate::unity_engine::{GameObject, MonoBehaviour};
use nalgebra::{Quaternion, Vector3};
use ordered_float::OrderedFloat;
use std::collections::BTreeMap;
use unity_mirror_macro_rs::{client_rpc, command, namespace, network_behaviour};

#[derive(Debug, PartialOrd, PartialEq, Clone, Default)]
#[allow(unused)]
pub enum CoordinateSpace {
    #[default]
    Local,
    World,
}

impl Into<CoordinateSpace> for metadata_network_transform_base::CoordinateSpace {
    fn into(self) -> CoordinateSpace {
        match self {
            metadata_network_transform_base::CoordinateSpace::Local => CoordinateSpace::Local,
            metadata_network_transform_base::CoordinateSpace::World => CoordinateSpace::World,
        }
    }
}

#[namespace(prefix = "Mirror")]
#[network_behaviour(parent(NetworkBehaviour), metadata(MetadataNetworkTransformBase))]
pub struct NetworkTransformBase {
    pub target: RevelWeak<Transform>,
    pub server_snapshots: BTreeMap<OrderedFloat<f64>, TransformSnapshot>,
    pub only_sync_on_change: bool,
    pub coordinate_space: CoordinateSpace,
    pub is_client_with_authority: bool,
    pub sync_position: bool,
    pub sync_rotation: bool,
    pub sync_scale: bool,
    pub compress_rotation: bool,
    pub interpolate_position: bool,
    pub interpolate_rotation: bool,
    pub interpolate_scale: bool,
    pub timeline_offset: bool,
    pub buffer_reset_multiplier: u32,
    pub send_interval_counter: u32,
    pub last_send_interval_time: f64,
}

// sync hooks
impl NetworkTransformBaseOnChangeCallback for NetworkTransformBase {}

// 远程调用
impl NetworkTransformBase {
    #[command(NetworkTransformBase, authority, rename = "CmdTeleport")]
    fn cmd_teleport(&self, destination: Vector3<f32>) {
        if self.sync_direction != SyncDirection::ClientToServer {
            return;
        }

        // self.on_teleport(destination);
        self.rpc_teleport(destination);
    }

    #[command(NetworkTransformBase, authority, rename = "CmdTeleport")]
    fn cmd_teleport_(&self, destination: Vector3<f32>, rotation: Quaternion<f32>) {
        if self.sync_direction != SyncDirection::ClientToServer {
            return;
        }

        // self.on_teleport_(destination, rotation);
        self.rpc_teleport_(destination, rotation);
    }

    #[client_rpc(include_owner, channel = TransportChannel::Reliable, rename = "RpcTeleport")]
    fn rpc_teleport(&self, destination: Vector3<f32>) {}

    #[client_rpc(include_owner, channel = TransportChannel::Reliable, rename = "RpcTeleport")]
    fn rpc_teleport_(&self, destination: Vector3<f32>, rotation: Quaternion<f32>) {}

    pub fn server_teleport(&self, destination: Vector3<f32>, rotation: Quaternion<f32>) {
        // self.on_teleport(destination, rotation);
        self.rpc_teleport_(destination, rotation);
    }

    #[client_rpc(include_owner, channel = TransportChannel::Reliable)]
    fn rpc_reset_state(&self) {}
}

impl NetworkTransformBase {
    pub fn send_interval_multiplier(&self) -> u32 {
        if self.sync_interval > 0.0 {
            let multiplier = self.sync_interval / NetworkServer.send_interval() as f32;
            if multiplier > 1.0 {
                return multiplier.round() as u32;
            }
        }
        1
    }

    pub fn time_stamp_adjustment(&self) -> f64 {
        NetworkServer.send_interval() * (self.send_interval_multiplier() - 1) as f64
    }

    pub fn offset(&self, time_stamp: f64) -> f64 {
        if self.timeline_offset {
            return NetworkServer.send_interval() * self.send_interval_multiplier() as f64;
        }
        0.0
    }

    pub fn get_position(&self) -> Vector3<f32> {
        if let Some(target) = self.target.get() {
            return match self.coordinate_space {
                CoordinateSpace::Local => target.local_position,
                CoordinateSpace::World => target.position,
            };
        }
        Vector3::new(0.0, 0.0, 0.0)
    }
    pub fn set_position(&self, value: Vector3<f32>) {
        if let Some(target) = self.target.get() {
            match self.coordinate_space {
                CoordinateSpace::Local => {
                    target.local_position = value;
                }
                CoordinateSpace::World => {
                    target.position = value;
                }
            }
        }
    }

    pub fn get_rotation(&self) -> Quaternion<f32> {
        if let Some(target) = self.target.get() {
            return match self.coordinate_space {
                CoordinateSpace::Local => target.local_rotation,
                CoordinateSpace::World => target.rotation,
            };
        }
        Quaternion::identity()
    }
    pub fn set_rotation(&self, value: Quaternion<f32>) {
        if let Some(target) = self.target.get() {
            match self.coordinate_space {
                CoordinateSpace::Local => {
                    target.local_rotation = value;
                }
                CoordinateSpace::World => {
                    target.rotation = value;
                }
            }
        }
    }

    pub fn get_scale(&self) -> Vector3<f32> {
        if let Some(target) = self.target.get() {
            return target.local_scale;
        }
        Vector3::new(1.0, 1.0, 1.0)
    }
    pub fn set_scale(&self, value: Vector3<f32>) {
        if let Some(target) = self.target.get() {
            target.local_scale = value;
        }
    }
}

// MonoBehaviour
impl MonoBehaviour for NetworkTransformBase {
    fn awake(&mut self) {
        // println!("Mirror: NetworkTransformBase Awake");
    }
    fn start(&mut self) {
        // println!("Mirror: NetworkTransformBase Start");
    }
    fn fixed_update(&mut self) {
        // println!("Mirror: NetworkTransformBase FixedUpdate");
    }
    fn update(&mut self) {
        // println!("Mirror: NetworkTransformBase Update");
    }
    fn late_update(&mut self) {
        // println!("Mirror: NetworkTransformBase LateUpdate");
    }
}

// TNetworkBehaviour
impl TNetworkBehaviour for NetworkTransformBase {
    fn new(
        weak_game_object: RevelWeak<GameObject>,
        metadata: &MetadataNetworkBehaviourWrapper,
    ) -> Self
    where
        Self: Sized,
    {
        let mut base = Self::default();

        {
            let config = metadata.get::<MetadataNetworkTransformBase>();
            if let Some(game) = weak_game_object.get() {
                if let Some(transform) = game.find_transform(&config.target.instance_id) {
                    base.target = transform;
                } else {
                    log::error!(
                        "Mirror: NetworkTransformBase target Transform with instance_id {} not found",
                        config.target.instance_id
                    );
                }
            }

            base.sync_position = config.sync_position;
            base.sync_rotation = config.sync_rotation;
            base.sync_scale = config.sync_scale;

            base.only_sync_on_change = config.only_sync_on_change;
            base.compress_rotation = config.compress_rotation;

            base.interpolate_position = config.interpolate_position;
            base.interpolate_rotation = config.interpolate_rotation;
            base.interpolate_scale = config.interpolate_scale;

            base.coordinate_space = config.coordinate_space.clone().into();
        }


        base
    }
}
