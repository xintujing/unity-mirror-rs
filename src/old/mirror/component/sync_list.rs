// use crate::mirror::component::sync_object::SyncObject;
// use crate::mirror::network_reader::NetworkReader;
// use crate::mirror::network_writer::NetworkWriter;
// use std::fmt::Debug;
//
// #[derive(Copy, Clone)]
// pub enum Operation {
//     Add,
//     Set,
//     Insert,
//     Remove,
//     Clear,
// }
//
// struct Change<T> {
//     operation: Operation,
//     index: usize,
//     item: T,
// }
//
// pub struct SyncList<T> {
//     id: String,
//
//     objects: Vec<T>,
//     // on_dirty: Box<dyn Fn(&str, u8)>,
//     on_change: Option<Box<dyn Fn(Operation, usize, &T)>>,
//     callback: Option<Box<dyn Fn(Operation, usize, &T, &T)>>,
//
//     changes: Vec<Change<T>>,
//     changes_ahead: i32,
// }
//
// impl<T> SyncList<T> {
//     pub fn is_read_only(&self) -> bool {
//         !self.is_writable()
//     }
//     pub fn len(&self) -> usize {
//         self.objects.len()
//     }
//
//     pub fn clear_changes(&mut self) {
//         self.changes.clear();
//     }
//
//     pub fn reset(&mut self) {
//         self.changes.clear();
//         self.changes_ahead = 0;
//         self.objects.clear();
//     }
//
//     fn add_operation(
//         &mut self,
//         operation: Operation,
//         index: usize,
//         old_item: T,
//         new_item: T,
//         check_access: bool,
//     ) {
//         if check_access && self.is_read_only() {
//             log::error!("SyncList can only be modified by the owner.");
//         }
//
//         let change = Change {
//             operation,
//             index,
//             item: new_item,
//         };
//
//         if self.is_recording() {
//             self.changes.push(change);
//             self.on_dirty(0)
//         }
//
//         match &operation {
//             Operation::Add => {
//                 self.on_add(index);
//                 if let Some(f) = &self.on_change {
//                     f(operation, index, &new_item)
//                 }
//                 if let Some(f) = &self.callback {
//                     f(operation, index, &old_item, &new_item)
//                 }
//             }
//             Operation::Set => {
//                 self.on_add(index);
//                 if let Some(f) = &self.on_change {
//                     f(operation, index, &new_item)
//                 }
//                 if let Some(f) = &self.callback {
//                     f(operation, index, &old_item, &new_item)
//                 }
//             }
//             Operation::Insert => {}
//             Operation::Remove => {}
//             Operation::Clear => {}
//         }
//     }
// }
//
// impl<T> SyncObject for SyncList<T> {
//     fn component_id(&self) -> &'static str {
//         &self.id
//     }
//     // fn on_dirty(&mut self) {
//     //     todo!()
//     // }
//
//     fn on_serialize_all(&self, writer: &mut NetworkWriter) {
//         todo!()
//     }
//
//     fn on_serialize_delta(&self, writer: &mut NetworkWriter) {
//         todo!()
//     }
//
//     fn on_deserialize_all(&mut self, reader: &mut NetworkReader) {
//         todo!()
//     }
//
//     fn on_deserialize_delta(&mut self, reader: &mut NetworkReader) {
//         todo!()
//     }
// }
