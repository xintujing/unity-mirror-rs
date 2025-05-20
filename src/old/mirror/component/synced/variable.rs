// use crate::mirror::network_reader::{DataTypeDeserializer, NetworkReader};
// use crate::mirror::network_writer::{DataTypeSerializer, NetworkWriter};
//
// #[derive(Default)]
// pub struct SyncVariable<T: DataTypeSerializer + DataTypeDeserializer> {
//     value: T,
//     hook: Option<fn(old_val: &T, new_val: &T)>,
//     index: u8,
// }
//
// impl<T: DataTypeSerializer + DataTypeDeserializer> SyncVariable<T> {
//     pub fn new(value: T, index: u8) -> Self {
//         Self {
//             value,
//             hook: None,
//             index,
//         }
//     }
//
//     pub fn hook(&mut self, hook: fn(old_val: &T, new_val: &T)) {
//         self.hook = Some(hook);
//     }
//     pub fn unhook(&mut self) {
//         self.hook = None
//     }
//
//     pub fn get(&self) -> &T {
//         &self.value
//     }
//
//     pub fn set(&mut self, value: T) {
//         if let Some(hook) = self.hook {
//             hook(&self.value, &value);
//         }
//         self.value = value;
//     }
//
//     pub fn serialize(
//         &self,
//         dirty_bit: &mut u64,
//         index_offset: u8,
//         writer: &mut NetworkWriter,
//         initial: bool,
//     ) {
//         if initial || *dirty_bit & (1u64 << (self.index + index_offset)) != 0 {
//             self.value.serialize(writer);
//         }
//     }
//
//     pub fn deserialize(
//         &mut self,
//         dirty_bit: &mut u64,
//         index_offset: u8,
//         reader: &mut NetworkReader,
//         initial: bool,
//     ) {
//         if initial || *dirty_bit & (1u64 << (self.index + index_offset)) != 0 {
//             self.set(T::deserialize(reader))
//         }
//     }
// }
