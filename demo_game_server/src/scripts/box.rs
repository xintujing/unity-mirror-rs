use std::any::Any;
use unity_mirror_rs::commons::revel_weak::RevelWeak;
use unity_mirror_rs::metadata_settings::mirror::network_behaviours::metadata_network_behaviour::MetadataNetworkBehaviourWrapper;
use unity_mirror_rs::mirror::{NetworkBehaviour, TNetworkBehaviour};
use unity_mirror_rs::unity_engine::{GameObject, MonoBehaviour, MonoBehaviourAny};
use unity_mirror_rs::unity_mirror_macro_rs::{namespace, network_behaviour};

#[namespace(rename = "Box")]
#[network_behaviour(
    parent(NetworkBehaviour),
    metadata(crate::backend_metadata::r#box::MetadataBox)
)]
pub struct BoxScript {}
impl MonoBehaviour for BoxScript {}
impl BoxScriptOnChangeCallback for BoxScript {}
impl TNetworkBehaviour for BoxScript {
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

// mod private_component_box_script {
//     use super::*;
//     #[namespace(rename = "Box")]
//     #[derive(Default, Debug, crate::unity_mirror_macro_rs::SyncState)]
//     pub struct BoxScript {
//         pub(super) ancestor: crate::commons::revel_weak::RevelWeak<Box<crate::mirror::NetworkBehaviour>>,
//         pub(super) parent: crate::commons::revel_weak::RevelWeak<Box<NetworkBehaviour>>,
//         pub(super) weak: crate::commons::revel_weak::RevelWeak<Box<Self>>,
//         obj_start_offset: u8,
//         var_start_offset: u8,
//     }
//     impl core::ops::Deref for BoxScript {
//         type Target = Box<NetworkBehaviour>;
//         fn deref(&self) -> &Self::Target { self.parent.get().unwrap() }
//     }
//     impl core::ops::DerefMut for BoxScript { fn deref_mut(&mut self) -> &mut Self::Target { self.parent.get().unwrap() } }
//     impl BoxScript {
//         pub fn factory(weak_game_object: crate::commons::revel_weak::RevelWeak<crate::unity_engine::GameObject>, metadata: &crate::metadata_settings::mirror::network_behaviours::metadata_network_behaviour::MetadataNetworkBehaviourWrapper, weak_network_behaviour: &mut crate::commons::revel_weak::RevelWeak<Box<crate::mirror::NetworkBehaviour>>, sync_object_offset: &mut u8, sync_var_offset: &mut u8) -> Vec<(crate::commons::revel_arc::RevelArc<Box<dyn crate::unity_engine::MonoBehaviour>>, std::any::TypeId)> {
//             let mut network_behaviour_chain = NetworkBehaviour::factory(weak_game_object.clone(), metadata, weak_network_behaviour, sync_object_offset, sync_var_offset);
//             let mut this = Self::new(weak_game_object.clone(), metadata);
//             {
//                 this.obj_start_offset = *sync_object_offset;
//                 this.var_start_offset = *sync_var_offset;
//                 *sync_object_offset += 0usize as u8;
//                 *sync_var_offset += 0usize as u8;
//             }
//             if let Some((arc_nb, _)) = network_behaviour_chain.first() { if let Some(weak_nb) = arc_nb.downgrade().downcast::<crate::mirror::NetworkBehaviour>() { this.ancestor = weak_nb.clone(); } }
//             if let Some((arc_nb, _)) = network_behaviour_chain.last() { if let Some(weak_nb) = arc_nb.downgrade().downcast::<NetworkBehaviour>() { this.parent = weak_nb.clone(); } }
//             { use crate::mirror::sync_object::SyncObject; }
//             { let config = metadata.get::<crate::backend_metadata::r#box::MetadataBox>(); }
//             let mut arc_this = crate::commons::revel_arc::RevelArc::new(Box::new(this) as Box<dyn crate::mirror::TNetworkBehaviour> as Box<dyn crate::unity_engine::MonoBehaviour>);
//             if let Some(weak_nb) = arc_this.downgrade().downcast::<Self>() { if let Some(mut this) = weak_nb.upgrade() { this.weak = weak_nb.clone(); } }
//             network_behaviour_chain.push((arc_this, std::any::TypeId::of::<Self>()));
//             network_behaviour_chain
//         }
//     }
//     #[ctor::ctor]
//     fn static_init() { crate::mirror::NetworkBehaviourFactory::register::<BoxScript>(BoxScript::factory); }
//     impl crate::mirror::NetworkBehaviourOnSerializer for BoxScript {
//         fn on_serialize(&mut self, writer: &mut crate::mirror::NetworkWriter, initial_state: bool) {
//             if let Some(mut parent) = self.parent.get() {
//                 use crate::mirror::NetworkBehaviourOnSerializer;
//                 parent.on_serialize(writer, initial_state);
//             } use crate::mirror::NetworkBehaviourSerializer;
//             self.serialize_sync_objects(writer, initial_state);
//             self.serialize_sync_vars(writer, initial_state);
//         }
//     }
//     impl crate::mirror::NetworkBehaviourOnDeserializer for BoxScript {
//         fn on_deserialize(&mut self, reader: &mut crate::mirror::NetworkReader, initial_state: bool) {
//             if let Some(mut parent) = self.parent.get() {
//                 use crate::mirror::NetworkBehaviourOnDeserializer;
//                 parent.on_deserialize(reader, initial_state);
//             } use crate::mirror::NetworkBehaviourDeserializer;
//             self.deserialize_sync_objects(reader, initial_state);
//             self.deserialize_sync_vars(reader, initial_state);
//         }
//     }
//     impl crate::mirror::NetworkBehaviourBase for BoxScript {
//         fn initialize(&mut self, index: u8, weak_identity: RevelWeak<Box<crate::mirror::NetworkIdentity>>) {
//             self.component_index = index;
//             self.network_identity = weak_identity;
//         }
//         fn is_dirty(&self) -> bool {
//             if let Some(ancestor) = self.ancestor.get() { return ancestor.is_dirty(); }
//             false
//         }
//         fn get_sync_direction(&self) -> &crate::mirror::SyncDirection { &self.sync_direction }
//         fn get_sync_mode(&self) -> &crate::mirror::SyncMode { &self.sync_mode }
//         fn clear_all_dirty_bits(&mut self) { if let Some(mut parent) = self.parent.get() { parent.clear_all_dirty_bits(); } }
//     }
//     impl crate::mirror::NetworkBehaviourSerializer for BoxScript {
//         fn serialize_sync_objects(&mut self, writer: &mut crate::mirror::NetworkWriter, initial_state: bool) { if initial_state { self.serialize_objects_all(writer); } else { self.serialize_sync_object_delta(writer); } }
//         fn serialize_objects_all(&mut self, writer: &mut crate::mirror::NetworkWriter) { use crate::mirror::sync_object::SyncObject; }
//         fn serialize_sync_object_delta(&mut self, writer: &mut crate::mirror::NetworkWriter) { use crate::mirror::sync_object::SyncObject; }
//         fn serialize_sync_vars(&mut self, writer: &mut crate::mirror::NetworkWriter, initial_state: bool) {
//             if 0usize == 0 { return; }
//             if let Some(mut network_behaviour) = self.ancestor.get() {
//                 use crate::mirror::DataTypeSerializer;
//                 let dirty_bits = network_behaviour.sync_var_dirty_bits;
//                 if initial_state { return; }
//                 writer.write_blittable_compress::<u64>(dirty_bits);
//             }
//         }
//     }
//     impl crate::mirror::NetworkBehaviourDeserializer for BoxScript {
//         fn deserialize_sync_objects(&mut self, reader: &mut crate::mirror::NetworkReader, initial_state: bool) { if initial_state { self.deserialize_objects_all(reader); } else { self.deserialize_sync_object_delta(reader); } }
//         fn deserialize_objects_all(&mut self, reader: &mut crate::mirror::NetworkReader) { use crate::mirror::sync_object::SyncObject; }
//         fn deserialize_sync_object_delta(&mut self, reader: &mut crate::mirror::NetworkReader) { use crate::mirror::sync_object::SyncObject; }
//         fn deserialize_sync_vars(&mut self, reader: &mut crate::mirror::NetworkReader, initial_state: bool) {
//             if 0usize == 0 { return; }
//             if let Some(mut network_behaviour) = self.ancestor.get() {
//                 use crate::mirror::DataTypeDeserializer;
//                 let mut dirty_bits = 0;
//                 if initial_state { return; }
//                 network_behaviour.sync_var_dirty_bits = reader.read_blittable::<u64>();
//                 dirty_bits = network_behaviour.sync_var_dirty_bits;
//             }
//         }
//     }
//     impl BoxScript {}
// }
// pub use private_component_box_script::BoxScript;
// trait BaseBoxScript: BoxScriptOnChangeCallback {}
// trait BoxScriptOnChangeCallback {}
// impl BaseBoxScript for BoxScript {}
// impl crate::mirror::TBaseNetworkBehaviour for BoxScript {}