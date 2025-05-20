use crate::metadata_settings::mirror::network_behaviours::metadata_network_behaviour::MetadataNetworkBehaviourWrapper;
use crate::mirror::component::component_basic::ComponentBasic;
use crate::mirror::component::component_clone::ComponentClone;
use crate::mirror::component::component_deserializer::ComponentDeserializer;
use crate::mirror::component::component_factory::ComponentFactory;
use crate::mirror::component::component_lifespan::ComponentLifespan;
use crate::mirror::component::component_serializer::ComponentSerializer;
use crate::mirror::component::component_type::ComponentType;

pub trait Component:
    Send
    + Sync
    + ComponentType
    + ComponentClone
    + ComponentSerializer
    + ComponentDeserializer
    + ComponentFactory
    + ComponentLifespan
    + ComponentBasic
{
    fn new(
        settings: &MetadataNetworkBehaviourWrapper,
        obj_start_offset: &mut u8,
        var_start_offset: &mut u8,
    ) -> Self
    where
        Self: Sized;
}
//
// #[macro_export]
// macro_rules! component {
//     ($namespace:expr, $component_ident:ident,$state_struct:path) => {
//         #[derive(Clone)]
//         #[allow(unused)]
//         pub struct $component_ident {
//             #[allow(unused)]
//             id: String,
//         }
//
//         impl crate::commons::namespace::Namespace for $component_ident {
//             fn get_namespace() -> &'static str {
//                 $namespace
//             }
//         }
//
//         impl crate::mirror::component::component_basic::ComponentBasic for $component_ident {
//             fn id(&self) -> String {
//                 self.id.clone()
//             }
//
//             fn parent(&self) -> Option<Box<dyn crate::mirror::component::component::Component>> {
//                 None
//             }
//
//             fn state_clear(&self) {
//                 if let Some(parent) = self.parent() {
//                     parent.clear_state();
//                 }
//                 let opt = Self::on_state_clear(&self.id);
//                 use crate::commons::namespace::Namespace;
//                 println!(
//                     "{}: remove state of {} is {}",
//                     self.id,
//                     Self::get_namespace(),
//                     opt.is_some()
//                 );
//             }
//         }
//     };
//
//     ($namespace:expr, $component_ident:ident, $parent:path,$state_struct:path) => {
//         #[derive(Clone)]
//         #[allow(unused)]
//         pub struct $component_ident {
//             id: String,
//             parent: $parent,
//         }
//
//         impl crate::commons::namespace::Namespace for $component_ident {
//             fn get_namespace() -> &'static str {
//                 $namespace
//             }
//         }
//
//         impl crate::mirror::component::component_basic::ComponentBasic for $component_ident {
//             fn id(&self) -> String {
//                 self.id.clone()
//             }
//
//             fn parent(&self) -> Option<Box<dyn crate::mirror::component::component::Component>> {
//                 Some(Box::new(self.parent.clone()))
//             }
//
//             fn state_clear(&self) {
//                 if let Some(parent) = self.parent() {
//                     parent.state_clear();
//                 }
//                 let opt = Self::on_state_clear(&self.id);
//                 use crate::commons::namespace::Namespace;
//                 println!(
//                     "{}: remove state of {} is {}",
//                     self.id,
//                     Self::get_namespace(),
//                     opt.is_some()
//                 );
//             }
//         }
//     };
// }
