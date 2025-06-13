use crate::commons::RevelWeak;
use crate::mirror::sync_object::SyncObject;
use crate::mirror::NetworkBehaviour;
use crate::mirror::{DataTypeDeserializer, NetworkReader};
use crate::mirror::{DataTypeSerializer, NetworkWriter};
use std::fmt::{Debug, Formatter};

#[derive(Debug, PartialEq, Copy, Clone)]
#[repr(u8)]
pub enum Operation {
    OpAdd = 0,
    OpSet = 1,
    OpInsert = 2,
    OpRemoveAt = 3,
    OpClear = 4,
}

impl Operation {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Operation::OpAdd),
            1 => Some(Operation::OpSet),
            2 => Some(Operation::OpInsert),
            3 => Some(Operation::OpRemoveAt),
            4 => Some(Operation::OpClear),
            _ => None,
        }
    }
}

pub struct Change<T: Clone + DataTypeSerializer + DataTypeDeserializer> {
    pub operation: Operation,
    pub index: usize,
    pub value: T,
}

pub struct SyncList<T: PartialEq + Clone + Default + DataTypeSerializer + DataTypeDeserializer> {
    network_behaviour: RevelWeak<Box<NetworkBehaviour>>,
    index: u8,
    value: Vec<T>,

    /// This is called for all changes to the List.
    /// <para>For OP_ADD and OP_INSERT, T is the NEW value of the entry.</para>
    /// <para>For OP_SET and OP_REMOVE, T is the OLD value of the entry.</para>
    /// <para>For OP_CLEAR, T is default.</para>
    pub on_change: Option<fn(Operation, usize, &T)>,
    /// <summary>
    /// This is called for all changes to the List.
    /// Parameters: Operation, index, oldItem, newItem.
    /// Sometimes we need both oldItem and newItem.
    /// Keep for compatibility since 10 years of projects use this.
    /// </summary>
    pub call_back: Option<fn(Operation, usize, &T, &T)>,

    changes: Vec<Change<T>>,
    change_ahead: usize,
}

impl<T: PartialEq + Clone + Default + DataTypeSerializer + DataTypeDeserializer> Debug
for SyncList<T>
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SyncList")
            // .field("index", &self.index)
            // .field("value", &self.value)
            // .field("on_change", &self.on_change)
            // .field("call_back", &self.call_back)
            // .field("changes", &self.changes)
            // .field("change_ahead", &self.change_ahead)
            .finish()
    }
}

impl<T: PartialEq + Clone + Default + DataTypeSerializer + DataTypeDeserializer> Default
for SyncList<T>
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T: PartialEq + Clone + Default + DataTypeSerializer + DataTypeDeserializer> SyncList<T> {
    // 遍历列表
    pub fn iter(&self) -> impl Iterator<Item=&T> {
        self.value.iter()
    }

    // 遍历列表
    pub fn iter_mut(&mut self) -> impl Iterator<Item=&mut T> {
        self.value.iter_mut()
    }

    // ********************************************************************

    pub fn count(&self) -> usize {
        self.value.len()
    }

    pub fn is_read_only(&self) -> bool {
        !self.is_writable()
    }

    pub fn clear_changes(&mut self) {
        self.changes.clear();
    }

    fn add_operation(
        &mut self,
        operation: Operation,
        item_index: usize,
        ov: &T,
        nv: &T,
        check_access: bool,
    ) {
        if check_access && self.is_read_only() {
            log::error!("Sync lists can only be modified by the owner.")
        }

        let change = Change {
            operation,
            index: item_index,
            value: nv.clone(),
        };

        self.changes.push(change);
        self.on_dirty();

        match operation {
            Operation::OpAdd | Operation::OpInsert | Operation::OpClear => {
                if let Some(on_change) = self.on_change {
                    on_change(operation, item_index, nv);
                }
                if let Some(call_back) = self.call_back {
                    call_back(operation, item_index, ov, nv);
                }
            }
            Operation::OpSet | Operation::OpRemoveAt => {
                if let Some(on_change) = self.on_change {
                    on_change(operation, item_index, ov);
                }
                if let Some(call_back) = self.call_back {
                    call_back(operation, item_index, ov, nv);
                }
            }
        }
    }

    pub fn add(&mut self, value: T) {
        self.add_operation(
            Operation::OpAdd,
            self.value.len(),
            &T::default(),
            &value,
            true,
        );
        self.value.push(value);
    }

    pub fn add_range(&mut self, values: Vec<T>) {
        for value in values {
            self.add(value);
        }
    }

    pub fn insert(&mut self, index: usize, value: T) {
        self.add_operation(Operation::OpInsert, index, &T::default(), &value, true);
        self.value.insert(index, value);
    }

    pub fn insert_range(&mut self, mut index: usize, values: Vec<T>) {
        for value in values {
            self.insert(index, value);
            index += 1;
        }
    }

    pub fn index_of(&self, value: &T) -> Option<usize> {
        self.value.iter().position(|x| x == value)
    }

    pub fn remove_at(&mut self, index: usize) {
        if index < self.value.len() {
            let ov = self.value.remove(index);
            self.add_operation(Operation::OpRemoveAt, index, &ov, &T::default(), true);
        }
    }

    pub fn remove(&mut self, value: &T) {
        if let Some(index) = self.index_of(value) {
            self.remove_at(index);
        }
    }

    pub fn clear(&mut self) {
        self.add_operation(Operation::OpClear, 0, &T::default(), &T::default(), true);
        self.value.clear();
    }

    pub fn contains(&self, item: &T) -> bool {
        self.value.contains(item)
    }
}

impl<T: PartialEq + Clone + Default + DataTypeSerializer + DataTypeDeserializer> SyncObject
for SyncList<T>
{
    type Item = Vec<T>;

    fn new() -> Self {
        Self {
            network_behaviour: Default::default(),
            index: 0,
            value: Self::Item::new(),
            on_change: None,
            call_back: None,
            changes: Vec::new(),
            change_ahead: 0,
        }
    }

    fn new_with_value(value: Self::Item) -> Self {
        Self {
            network_behaviour: Default::default(),
            index: 0,
            value,
            on_change: None,
            call_back: None,
            changes: Vec::new(),
            change_ahead: 0,
        }
    }

    fn set_network_behaviour(&mut self, network_behaviour: RevelWeak<Box<NetworkBehaviour>>) {
        self.network_behaviour = network_behaviour;
    }

    fn network_behaviour(&self) -> &RevelWeak<Box<NetworkBehaviour>> {
        &self.network_behaviour
    }

    fn set_index(&mut self, index: u8) {
        self.index = index;
    }

    fn index(&self) -> u8 {
        self.index
    }

    fn clear_changes(&mut self) {
        self.changes.clear();
    }

    fn on_serialize_all(&self, writer: &mut NetworkWriter) {
        writer.write_blittable::<u32>(self.count() as u32);

        for value in self.value.to_vec() {
            value.serialize(writer);
        }

        writer.write_blittable::<u32>(self.changes.len() as u32);
    }

    fn on_serialize_delta(&self, writer: &mut NetworkWriter) {
        writer.write_blittable::<u32>(self.changes.len() as u32);

        for change in self.changes.iter() {
            writer.write_blittable::<u8>(change.operation as u8);

            match change.operation {
                Operation::OpAdd => {
                    change.value.clone().serialize(writer);
                }
                Operation::OpSet | Operation::OpInsert => {
                    writer.write_blittable::<u32>(change.index as u32);
                    change.value.clone().serialize(writer);
                }
                Operation::OpRemoveAt => {
                    writer.write_blittable::<u32>(change.index as u32);
                }
                Operation::OpClear => {}
            }
        }
    }

    fn on_deserialize_all(&mut self, reader: &mut NetworkReader) {
        let count = reader.read_blittable::<u32>() as usize;

        self.value.clear();
        self.changes.clear();

        for _ in 0..count {
            let obj = T::deserialize(reader);
            self.value.push(obj);
        }

        self.change_ahead = reader.read_blittable::<u32>() as usize;
    }

    fn on_deserialize_delta(&mut self, reader: &mut NetworkReader) {
        let changes_count = reader.read_blittable::<u32>() as usize;

        for _ in 0..changes_count {
            let op = reader.read_blittable::<u8>();
            if let Some(operation) = Operation::from_u8(op) {
                let apply = self.change_ahead == 0;
                let mut ov = T::default();
                let mut nv = T::default();

                match operation {
                    Operation::OpAdd => {
                        nv = T::deserialize(reader);
                        if apply {
                            self.add_operation(Operation::OpAdd, self.count(), &ov, &nv, false);
                            self.value.push(nv);
                        }
                    }
                    Operation::OpSet => {
                        let index = reader.read_blittable::<u32>() as usize;
                        nv = T::deserialize(reader);
                        if apply {
                            self.add_operation(Operation::OpSet, index, &ov, &nv, false);
                            self.value[index] = nv;
                        }
                    }
                    Operation::OpInsert => {
                        let index = reader.read_blittable::<u32>() as usize;
                        nv = T::deserialize(reader);
                        if apply {
                            self.add_operation(Operation::OpInsert, index, &ov, &nv, false);
                            self.value.insert(index, nv);
                        }
                    }
                    Operation::OpRemoveAt => {
                        let index = reader.read_blittable::<u32>() as usize;
                        if apply {
                            ov = self.value.remove(index);
                            self.add_operation(Operation::OpRemoveAt, index, &ov, &nv, false);
                        }
                    }
                    Operation::OpClear => {
                        if apply {
                            self.add_operation(Operation::OpClear, 0, &ov, &nv, false);
                            self.value.clear();
                        }
                    }
                }

                if !apply {
                    self.change_ahead -= 1;
                }
            }
        }
    }

    fn reset(&mut self) {
        self.changes.clear();
        self.change_ahead = 0;
        self.value.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn on_sync_list1_change(operation: Operation, index: usize, value: &i32) {
        println!(
            "1 on_sync_list_change: {:?}, {}, {}",
            operation, index, value
        );
    }

    fn on_sync_list2_change(operation: Operation, index: usize, value: &i32) {
        println!(
            "2 on_sync_list_change: {:?}, {}, {}",
            operation, index, value
        );
    }

    // #[test]
    // fn test_sync_list() {
    //     let mut sync_list1: SyncList<i32> = SyncList::new();
    //     let mut sync_list2: SyncList<i32> = SyncList::new();
    //
    //     let mut writer = NetworkWriter::new();
    //     let mut reader = NetworkReader::new()
    //
    //     sync_list1.on_change = Some(on_sync_list1_change);
    //     sync_list2.on_change = Some(on_sync_list2_change);
    //
    //     // ser add
    //     sync_list1.add(4);
    //     sync_list1.iter(|item| {
    //         println!("1 sync_list1: {}", item);
    //     });
    //     sync_list1.on_serialize_delta(&mut writer);
    //     sync_list1.clear_changes();
    //
    //     let data = writer.to_bytes();
    //     reader.set_bytes(data);
    //     // de ser add
    //     sync_list2.on_deserialize_delta(&mut reader);
    //     sync_list2.iter(|item| {
    //         println!("1 sync_list2: {}", item);
    //     });
    //
    //     // ****************************************************
    //
    //     let mut writer = NetworkWriter::new();
    //     let mut reader = NetworkReader::new();
    //     // ser remove
    //     sync_list1.remove(&4);
    //     sync_list1.iter(|item| {
    //         println!("2 sync_list1: {}", item);
    //     });
    //     sync_list1.on_serialize_delta(&mut writer);
    //     sync_list1.clear_changes();
    //     let data = writer.to_bytes();
    //     reader.set_bytes(data);
    //     // de ser remove
    //     sync_list2.on_deserialize_delta(&mut reader);
    //     sync_list2.iter(|item| {
    //         println!("2 sync_list2: {}", item);
    //     });
    // }

    // #[test]
    // fn test_add1() {
    //     let mut sync_list: SyncList<i32> = SyncList::new();
    //     sync_list.add(1);
    //
    //     let mut writer = NetworkWriter::new();
    //     sync_list.on_serialize_delta(&mut writer);
    //     let data = writer.to_bytes();
    //     println!("Serialized data: {:?}", data);
    // }
}
