use crate::commons::revel_arc::RevelArc;
use crate::commons::revel_weak::RevelWeak;
use crate::metadata_settings::mirror::network_behaviours::metadata_network_behaviour::{
    MetadataNetworkBehaviour, MetadataNetworkBehaviourWrapper, MetadataSyncDirection,
    MetadataSyncMode,
};
use crate::mirror::messages::message::MessageSerializer;
use crate::mirror::messages::rpc_message::RpcMessage;
use crate::mirror::network_connection::NetworkConnection;
use crate::mirror::network_reader::NetworkReader;
use crate::mirror::network_writer::NetworkWriter;
use crate::mirror::network_writer_pool::NetworkWriterPool;
use crate::mirror::transport::TransportChannel;
use crate::mirror::NetworkIdentity;
use crate::unity_engine::{GameObject, MonoBehaviour};
use crate::unity_engine::{Time, Transform};
use std::any::TypeId;
use unity_mirror_macro::namespace;

#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub enum SyncDirection {
    #[default]
    ServerToClient,
    ClientToServer,
}

impl Into<SyncDirection> for MetadataSyncDirection {
    fn into(self) -> SyncDirection {
        match &self {
            MetadataSyncDirection::ServerToClient => SyncDirection::ServerToClient,
            MetadataSyncDirection::ClientToServer => SyncDirection::ClientToServer,
        }
    }
}

#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub enum SyncMode {
    #[default]
    Observers,
    Owner,
}

impl Into<SyncMode> for MetadataSyncMode {
    fn into(self) -> SyncMode {
        match &self {
            MetadataSyncMode::Observers => SyncMode::Observers,
            MetadataSyncMode::Owner => SyncMode::Owner,
        }
    }
}

#[namespace(prefix = "Mirror")]
#[derive(Default)]
pub struct NetworkBehaviour {
    sync_direction: SyncDirection,
    sync_mode: SyncMode,
    sync_interval: f32,
    last_sync_time: f64,

    net_id: u32,
    component_index: u8,

    network_identity: RevelWeak<NetworkIdentity>,
    pub game_object: RevelWeak<GameObject>,
    transform: RevelWeak<Transform>,

    pub sync_var_dirty_bits: u64,
    pub sync_object_dirty_bits: u64,
}

impl NetworkBehaviour {
    pub fn is_server(&self) -> bool {
        if let Some(network_identity) = self.network_identity.get() {
            return network_identity.is_server;
        }
        false
    }

    pub fn is_client(&self) -> bool {
        if let Some(network_identity) = self.network_identity.get() {
            return network_identity.is_client;
        }
        false
    }

    pub fn is_server_only(&self) -> bool {
        if let Some(network_identity) = self.network_identity.get() {
            return network_identity.is_server_only();
        }
        false
    }

    pub fn is_client_only(&self) -> bool {
        if let Some(network_identity) = self.network_identity.get() {
            return network_identity.is_client_only();
        }
        false
    }

    pub fn is_owned(&self) -> bool {
        if let Some(network_identity) = self.network_identity.get() {
            return network_identity.is_owned;
        }
        false
    }
}

#[ctor::ctor]
fn static_init() {
    crate::mirror::network_behaviour_factory::NetworkBehaviourFactory::register::<NetworkBehaviour>(
        NetworkBehaviour::factory,
    );
}

impl NetworkBehaviour {
    pub fn factory(
        weak_game_object: RevelWeak<GameObject>,
        metadata: &MetadataNetworkBehaviourWrapper,
        weak_network_behaviour: &mut RevelWeak<Box<NetworkBehaviour>>,
        _: &mut u8,
        _: &mut u8,
    ) -> Vec<(RevelArc<Box<dyn MonoBehaviour>>, TypeId)> {
        let mut network_behaviour = Self::new(&weak_game_object, metadata);

        {
            let config = metadata.get::<MetadataNetworkBehaviour>();
            network_behaviour.sync_direction = config.sync_direction.clone().into();
            network_behaviour.sync_mode = config.sync_mode.clone().into();
            network_behaviour.sync_interval = config.sync_interval;
            network_behaviour.game_object = weak_game_object.clone();
            if let Some(game_object) = weak_game_object.get() {
                network_behaviour.transform = game_object.transform.downgrade();
            }
        }

        let arc_box_network_behaviour =
            RevelArc::new(Box::new(network_behaviour) as Box<dyn MonoBehaviour>);

        if let Some(value) = arc_box_network_behaviour
            .downgrade()
            .downcast::<NetworkBehaviour>()
        {
            *weak_network_behaviour = value.clone();
        }

        vec![(arc_box_network_behaviour, TypeId::of::<NetworkBehaviour>())]
    }
}

impl MonoBehaviour for NetworkBehaviour {}

impl TNetworkBehaviour for NetworkBehaviour {
    fn new(_: &RevelWeak<GameObject>, _: &MetadataNetworkBehaviourWrapper) -> Self
    where
        Self: Sized,
    {
        Self::default()
    }
}

impl NetworkBehaviourBase for NetworkBehaviour {
    fn is_dirty(&self) -> bool {
        (self.sync_var_dirty_bits | self.sync_object_dirty_bits) != 0u64
            && Time::unscaled_time_f64() - self.last_sync_time > self.sync_interval as f64
    }

    fn get_sync_direction(&self) -> &SyncDirection {
        &self.sync_direction
    }

    fn get_sync_mod(&self) -> &SyncMode {
        &self.sync_mode
    }
    fn clear_all_dirty_bits(&mut self) {
        self.sync_var_dirty_bits = 0;
        self.sync_object_dirty_bits = 0;
    }
}

impl NetworkBehaviourOnSerializer for NetworkBehaviour {
    fn on_serialize(&mut self, writer: &mut NetworkWriter, initial_state: bool) {
        self.serialize_sync_objects(writer, initial_state);
        self.serialize_sync_vars(writer, initial_state);
    }
}

impl NetworkBehaviourSerializer for NetworkBehaviour {
    fn serialize_sync_objects(&mut self, writer: &mut NetworkWriter, initial_state: bool) {
        if initial_state {
            self.serialize_objects_all(writer);
        } else {
            writer.write_blittable::<u64>(self.sync_object_dirty_bits);
            self.serialize_sync_object_delta(writer);
        }
    }
}

impl NetworkBehaviourOnDeserializer for NetworkBehaviour {
    fn on_deserialize(&mut self, reader: &mut NetworkReader, initial_state: bool) {
        self.deserialize_sync_objects(reader, initial_state);
        self.deserialize_sync_vars(reader, initial_state);
    }
}

impl NetworkBehaviourDeserializer for NetworkBehaviour {
    fn deserialize_sync_objects(&mut self, reader: &mut NetworkReader, initial_state: bool) {
        if initial_state {
            self.deserialize_objects_all(reader);
        } else {
            self.sync_object_dirty_bits = reader.read_blittable::<u64>();
            self.deserialize_sync_object_delta(reader);
        }
    }
}

pub trait TBaseNetworkBehaviour: TNetworkBehaviour {}
pub trait TNetworkBehaviour:
    MonoBehaviour + NetworkBehaviourBase + NetworkBehaviourSerializer + NetworkBehaviourDeserializer
{
    fn new(
        weak_game_object: &RevelWeak<GameObject>,
        metadata: &MetadataNetworkBehaviourWrapper,
    ) -> Self
    where
        Self: Sized;
    fn on_start_server(&mut self) {}
    fn on_stop_server(&mut self) {}
}
pub trait NetworkBehaviourBase {
    fn is_dirty(&self) -> bool;
    fn get_sync_direction(&self) -> &SyncDirection;
    fn get_sync_mod(&self) -> &SyncMode;
    fn clear_all_dirty_bits(&mut self);
}
pub trait NetworkBehaviourOnSerializer {
    fn on_serialize(&mut self, writer: &mut NetworkWriter, initial_state: bool) {}
}
pub trait NetworkBehaviourSerializer: NetworkBehaviourOnSerializer {
    fn serialize_sync_objects(&mut self, writer: &mut NetworkWriter, initial_state: bool) {}
    fn serialize_objects_all(&mut self, writer: &mut NetworkWriter) {}
    fn serialize_sync_object_delta(&mut self, writer: &mut NetworkWriter) {}
    fn serialize_sync_vars(&mut self, writer: &mut NetworkWriter, initial_state: bool) {}
}

pub trait NetworkBehaviourOnDeserializer {
    fn on_deserialize(&mut self, reader: &mut NetworkReader, initial_state: bool) {}
}

pub trait NetworkBehaviourDeserializer: NetworkBehaviourOnDeserializer {
    fn deserialize_sync_objects(&mut self, reader: &mut NetworkReader, initial_state: bool) {}
    fn deserialize_objects_all(&mut self, reader: &mut NetworkReader) {}
    fn deserialize_sync_object_delta(&mut self, reader: &mut NetworkReader) {}
    fn deserialize_sync_vars(&mut self, reader: &mut NetworkReader, initial_state: bool) {}
}

impl NetworkBehaviour {
    pub fn send_rpc_internal(
        &self,
        _function_full_name: &str,
        function_hash_code: u16,
        writer: &mut NetworkWriter,
        channel_id: TransportChannel,
        include_owner: bool,
    ) {
        if let Some(network_identity) = self.network_identity.get() {
            if network_identity.observers.is_empty() {
                return;
            }

            let mut message = RpcMessage::new(
                self.net_id,
                self.component_index,
                function_hash_code,
                writer.to_vec(),
            );

            NetworkWriterPool::get_return(|writer| {
                MessageSerializer::serialize(&mut message, writer);

                for observer in network_identity.observers.iter() {
                    if let (Some(observer), Some(connection)) =
                        (observer.get(), network_identity.connection().get())
                    {
                        let is_owner = observer.id == connection.id;

                        if (!is_owner || include_owner) && observer.is_ready {
                            observer.send_message(&mut message.clone(), channel_id.into());
                        }
                    }
                }
            });
        }
    }

    pub fn send_target_rpc_internal(
        &self,
        mut target_rpc_conn: Option<RevelArc<NetworkConnection>>,
        function_full_name: &str,
        function_hash_code: u16,
        writer: &mut NetworkWriter,
        channel_id: TransportChannel,
    ) {
        // rpc消息
        let mut message = RpcMessage::new(0, 0, function_hash_code, writer.to_vec());
        // 找出需要的数据

        if target_rpc_conn.is_none() {
            if let Some(weak_connection) = self.connection_to_client() {
                target_rpc_conn = weak_connection.upgrade();
            }
        }

        if target_rpc_conn.is_none() {
            log::error!(
                "TargetRPC '{}' can't be sent because it was given a null connection. Make sure {} is owned by a connection, or if you pass a connection manually then make sure it's not null. For example, TargetRpcs can be called on Player/Pet which are owned by a connection. However, they can not be called on Monsters/Npcs which don't have an owner connection.",
                self.game_object.get().unwrap().name,
                function_full_name,
            );
            return;
        }

        message.component_index = self.component_index;
        message.net_id = self.net_id;

        let mut connection = target_rpc_conn.unwrap();

        if connection.is_ready {
            NetworkWriterPool::get_return(|writer| {
                MessageSerializer::serialize(&mut message, writer);
                connection.send_message(&mut message, channel_id);
            });
        }
    }
}

impl NetworkBehaviour {
    pub fn connection_to_client(&self) -> Option<RevelWeak<NetworkConnection>> {
        if let Some(network_identity) = self.network_identity.get() {
            return Some(network_identity.connection());
        }
        None
    }
}
