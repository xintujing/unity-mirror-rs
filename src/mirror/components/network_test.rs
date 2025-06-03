#![allow(unused)]
use crate::commons::revel_weak::RevelWeak;
use crate::metadata_settings::mirror::network_behaviours::metadata_network_animator::MetadataNetworkAnimator;
use crate::metadata_settings::mirror::network_behaviours::metadata_network_behaviour::MetadataNetworkBehaviourWrapper;
use crate::mirror::network_behaviour::{
    NetworkBehaviourOnDeserializer, NetworkBehaviourOnSerializer, NetworkBehaviourSerializer,
    TNetworkBehaviour,
};
use crate::mirror::network_reader::NetworkReader;
use crate::mirror::network_writer::NetworkWriter;
use crate::mirror::sync_list::SyncList;
use crate::mirror::NetworkBehaviour;
use crate::unity_engine::{GameObject, MonoBehaviour};
use unity_mirror_macro::{
    ancestor_on_deserialize, ancestor_on_serialize, namespace, network_behaviour,
};

#[namespace(prefix = "Mirror")]
#[network_behaviour(
    parent(NetworkBehaviour),
    metadata(MetadataNetworkAnimator),
    not_impl_nos
)]
pub struct NetworkTest {
    #[sync_variable]
    pub sync_var_01: f32,
    #[sync_object]
    pub sync_obj_01: SyncList<i32>,
}

impl MonoBehaviour for NetworkTest {
    fn awake(&mut self) {
        // println!("Mirror: NetworkAnimator Awake");
    }
}

impl TNetworkBehaviour for NetworkTest {
    fn new(
        weak_game_object: &RevelWeak<GameObject>,
        metadata: &MetadataNetworkBehaviourWrapper,
    ) -> Self
    where
        Self: Sized,
    {
        let mut test = Self::default();

        match test.ancestor.get() {
            None => {}
            Some(network_behaviour) => {
                match network_behaviour.game_object.get() {
                    None => {}
                    Some(value) => {
                        println!("Mirror: Got another game object");
                    }
                }
                println!("Mirror: Got another game object");
            }
        }

        test.set_sync_var_01(888.0);

        test
    }
}

impl NetworkTestOnChangeCallback for NetworkTest {}

impl NetworkBehaviourOnSerializer for NetworkTest {
    #[ancestor_on_serialize]
    fn on_serialize(&mut self, writer: &mut NetworkWriter, initial_state: bool) {
        println!(
            "NetworkTest: on_serialize called with initial_state: {}",
            initial_state
        );
    }
}
impl NetworkBehaviourOnDeserializer for NetworkTest {
    #[ancestor_on_deserialize]
    fn on_deserialize(&mut self, reader: &mut NetworkReader, initial_state: bool) {
        println!(
            "NetworkTest: on_deserialize called with initial_state: {}",
            initial_state
        );
    }
}

#[test]
fn test_network_test() {
    let mut network_test = NetworkTest::default();

    network_test.sync_obj_01.on_change = Some(|a, b, c| {
        println!("SyncList changed: {:?} {:?} {:?}", a, b, c);
    });

    network_test.set_sync_var_01(1.0);
    println!("{}", network_test.get_sync_var_01());

    network_test.sync_obj_01.add(1);
    network_test.sync_obj_01.add(2);
    network_test.sync_obj_01.iter(|x| {
        println!("{}", x);
    })

    // let writer = &mut NetworkWriter::new();
    // network_test.serialize_sync_vars(writer, false);
    // println!("{:?}", writer.to_array_segment());
}
