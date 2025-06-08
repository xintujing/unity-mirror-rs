use crate::commons::revel_weak::RevelWeak;
use crate::metadata_settings::mirror::network_behaviours::metadata_network_behaviour::MetadataNetworkBehaviourWrapper;
use crate::metadata_settings::mirror::network_behaviours::metadata_network_transform_unreliable::MetadataNetworkTransformUnreliable;
use crate::mirror::components::network_transform::network_transform_base::NetworkTransformBase;
use crate::mirror::components::network_transform::transform_snapshot::TransformSnapshot;
use crate::mirror::components::network_transform::transform_sync_data::SyncData;
use crate::mirror::TNetworkBehaviour;
use crate::mirror::NetworkReader;
use crate::mirror::NetworkWriter;
use crate::mirror::transport::TransportChannel;
use crate::mirror::{
    NetworkBehaviourOnDeserializer, NetworkBehaviourOnSerializer, SyncDirection,
    TBaseNetworkBehaviour,
};
use crate::unity_engine::Time;
use crate::unity_engine::{GameObject, MonoBehaviour};
use nalgebra::{Quaternion, Vector3};
use unity_mirror_macro_rs::{client_rpc, command, namespace, network_behaviour};

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
    #[command(NetworkTransformUnreliable, authority)]
    fn cmd_client_to_server_sync(&self, sync_data: SyncData) {
        if self.sync_direction != SyncDirection::ClientToServer {
            return;
        }

        self.rpc_server_to_client_sync(sync_data);
    }

    // RpcServerToClientSync(SyncData syncData)
    #[client_rpc(include_owner, channel = TransportChannel::Unreliable)]
    fn rpc_server_to_client_sync(&self, sync_data: SyncData) {}
}

impl MonoBehaviour for NetworkTransformUnreliable {
    fn awake(&mut self) {
        // if let Some(parent) = self.parent.get() {
        //     parent.awake();
        // }
        // println!("Mirror: NetworkTransformUnreliable Awake");
    }
    fn start(&mut self) {
        // println!("Mirror: NetworkTransformUnreliable Start");
    }
    fn fixed_update(&mut self) {
        // let elapsed = Time::unscaled_time().elapsed();
        // println!(
        //     "Mirror: NetworkTransformUnreliable FixedUpdate {:?}",
        //     elapsed
        // );
    }
    fn update(&mut self) {
        // if let Some(parent) = self.parent.get() {
        //     parent.update();
        // }
        // let elapsed = Time::unscaled_time().elapsed();
        // println!("Mirror: NetworkTransformUnreliable Update {:?}", elapsed);

        let game_object = &self.game_object;

        // let game_object = root_game_object.get().unwrap();
        let weak_game_object = game_object
            .get()
            .unwrap()
            .try_get_component::<NetworkTransformUnreliable>()
            .unwrap();

        let x = weak_game_object
            .downcast::<NetworkTransformUnreliable>()
            .unwrap();

        let x1 = x.get().unwrap();

        // let weak_network_transform_unreliable =
        //     weak_game_object.to::<NetworkTransformUnreliable>();
        // let x = weak_network_transform_unreliable.get().unwrap();
        // println!("{}", x1.buffer_reset_multiplier);
    }
    fn late_update(&mut self) {
        let elapsed = Time::unscaled_time().elapsed();

        // println!(
        //     "Mirror: NetworkTransformUnreliable LateUpdate {:?}",
        //     elapsed
        // );
    }
}

impl TNetworkBehaviour for NetworkTransformUnreliable {
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

impl NetworkBehaviourOnSerializer for NetworkTransformUnreliable {
    fn on_serialize(&mut self, writer: &mut NetworkWriter, initial_state: bool) {
        // TODO
        if initial_state {
            if self.sync_position {
                writer.write_blittable(Vector3::default())
            }
            if self.sync_rotation {
                writer.write_blittable(Quaternion::default())
            }
            if self.sync_scale {
                writer.write_blittable(Vector3::default())
            }
        }
    }
}
impl NetworkBehaviourOnDeserializer for NetworkTransformUnreliable {
    fn on_deserialize(&mut self, reader: &mut NetworkReader, initial_state: bool) {
        // TODO
        if initial_state {
            if self.sync_position {}
            if self.sync_rotation {}
            if self.sync_scale {}
        }
    }
}
