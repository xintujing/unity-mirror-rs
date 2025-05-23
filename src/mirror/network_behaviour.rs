use crate::commons::revel_arc::RevelArc;
use crate::commons::revel_weak::RevelWeak;
use crate::metadata_settings::mirror::network_behaviours::metadata_network_behaviour::{
    MetadataNetworkBehaviour, MetadataNetworkBehaviourWrapper, MetadataSyncDirection,
    MetadataSyncMode,
};
use crate::mirror::network_behaviour_trait::{
    NetworkBehaviourDeserializer, NetworkBehaviourOnSerializer, NetworkBehaviourSerializer,
    NetworkBehaviourT,
};
use crate::mirror::network_reader::NetworkReader;
use crate::mirror::network_writer::NetworkWriter;
use crate::mirror::NetworkIdentity;
use crate::unity_engine::Transform;
use crate::unity_engine::{GameObject, MonoBehaviour};
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
        let mut network_behaviour = Self::new(metadata);

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

impl MonoBehaviour for NetworkBehaviour {
    fn awake(&mut self) {
        println!("NetworkBehaviour: awake");
    }
}

impl NetworkBehaviourT for NetworkBehaviour {
    fn new(_: &MetadataNetworkBehaviourWrapper) -> Self
    where
        Self: Sized,
    {
        Self::default()
    }
}

impl NetworkBehaviourOnSerializer for NetworkBehaviour {}

impl NetworkBehaviourSerializer for NetworkBehaviour {
    fn serialize_sync_objects(&mut self, writer: &mut NetworkWriter, initial_state: bool) {
        if initial_state {
            self.serialize_objects_all(writer);
        } else {
            writer.write_blittable::<u64>(self.sync_object_dirty_bits);
            self.serialize_sync_object_delta(writer);
        }
    }
    fn clear_all_dirty_bits(&mut self) {
        self.sync_var_dirty_bits = 0;
        self.sync_object_dirty_bits = 0;
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
