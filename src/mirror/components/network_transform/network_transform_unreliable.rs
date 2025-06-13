use crate::commons::Object;
use crate::commons::RevelArc;
use crate::commons::RevelWeak;
use crate::metadata_settings::MetadataNetworkBehaviourWrapper;
use crate::metadata_settings::MetadataNetworkTransformUnreliable;
use crate::mirror::components::*;
use crate::mirror::*;
use crate::unity_engine::{GameObject, MonoBehaviour};
use crate::SyncState;
use crate::{client_rpc, command, namespace, network_behaviour};
use nalgebra::{Quaternion, Vector3};

#[namespace(prefix = "Mirror")]
#[network_behaviour(
    parent(NetworkTransformBase),
    metadata(MetadataNetworkTransformUnreliable),
    not_impl_nos
)]
pub struct NetworkTransformUnreliable {
    pub buffer_reset_multiplier: f32,
    pub position_sensitivity: f32,
    pub rotation_sensitivity: f32,
    pub scale_sensitivity: f32,
    pub send_interval_counter: u32,
    pub last_send_interval_time: f64,

    pub last_snapshot: TransformSnapshot,
    pub cached_snapshot_comparison: bool,
    pub cached_changed_comparison: u8,
    pub has_sent_unchanged_position: bool,
}

impl NetworkTransformUnreliableOnChangeCallback for NetworkTransformUnreliable {}

impl NetworkTransformUnreliable {
    // CmdClientToServerSync(SyncData syncData)
    #[command(NetworkTransformUnreliable)]
    fn cmd_client_to_server_sync(&self, sync_data: SyncData) {
        if self.sync_direction != SyncDirection::ClientToServer {
            return;
        }

        self.rpc_server_to_client_sync(sync_data);
    }

    // RpcServerToClientSync(SyncData syncData)
    #[client_rpc(channel = TransportChannel::Unreliable)]
    fn rpc_server_to_client_sync(&self, sync_data: SyncData) {}
}

impl MonoBehaviour for NetworkTransformUnreliable {
    fn awake(&mut self) {}
    fn start(&mut self) {}
    fn fixed_update(&mut self) {}
    fn update(&mut self) {}
    fn late_update(&mut self) {}
}

impl TNetworkBehaviour for NetworkTransformUnreliable {
    fn new(
        _weak_game_object: RevelWeak<GameObject>,
        metadata: &MetadataNetworkBehaviourWrapper,
    ) -> Self
    where
        Self: Sized,
    {
        let mut unreliable = Self::default();
        {
            let config = metadata.get::<MetadataNetworkTransformUnreliable>();
            unreliable.buffer_reset_multiplier = config.buffer_reset_multiplier;
            unreliable.position_sensitivity = config.position_sensitivity;
            unreliable.rotation_sensitivity = config.rotation_sensitivity;
            unreliable.scale_sensitivity = config.scale_sensitivity;
        }

        unreliable
    }
}

impl NetworkBehaviourOnSerializer for NetworkTransformUnreliable {
    fn on_serialize(&mut self, writer: &mut NetworkWriter, initial_state: bool) {
        if initial_state {
            if self.sync_position {
                writer.write_blittable(self.get_position())
            }
            if self.sync_rotation {
                writer.write_blittable(self.get_rotation())
            }
            if self.sync_scale {
                writer.write_blittable(self.get_scale())
            }
        }
    }
}
impl NetworkBehaviourOnDeserializer for NetworkTransformUnreliable {
    fn on_deserialize(&mut self, reader: &mut NetworkReader, initial_state: bool) {
        if initial_state {
            if self.sync_position {
                let position: Vector3<f32> = reader.read_blittable();
                self.set_position(position);
            }
            if self.sync_rotation {
                let rotation: Quaternion<f32> = reader.read_blittable();
                self.set_rotation(rotation);
            }
            if self.sync_scale {
                let scale: Vector3<f32> = reader.read_blittable();
                self.set_scale(scale);
            }
        }
    }
}
