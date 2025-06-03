// use crate::mirror::component::synced::HookFn;
// use crate::mirror::network_reader::{DataTypeDeserializer, NetworkReader};
// use crate::mirror::network_writer::{DataTypeSerializer, NetworkWriter};
//
// #[derive(Default)]
// pub struct SyncObject<T: DataTypeSerializer + DataTypeDeserializer> {
//     value: T,
//     hook: Option<HookFn<T>>,
//     index: u8,
// }
//
// #[allow(unused)]
// impl<T: DataTypeSerializer + DataTypeDeserializer> SyncObject<T> {
//     pub fn new(value: T, index: u8) -> Self {
//         Self {
//             value,
//             hook: None,
//             index,
//         }
//     }
//
//     pub fn hook(&mut self, hook: HookFn<T>) {
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
//         if let Some(hook) = &self.hook {
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
