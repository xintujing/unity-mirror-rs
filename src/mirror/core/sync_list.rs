use crate::mirror::core::network_reader::{NetworkReader, NetworkReaderTrait};
use crate::mirror::core::network_writer::{NetworkWriter, NetworkWriterTrait};
use crate::mirror::core::sync_object::SyncObject;

#[repr(u8)]
#[derive(Debug, Copy, Clone)]
pub enum Operation {
    OpAdd,
    OpSet,
    OpInsert,
    OpRemoveAt,
    OpClear,
    None,
}

impl Operation {
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Operation::OpAdd,
            1 => Operation::OpSet,
            2 => Operation::OpInsert,
            3 => Operation::OpRemoveAt,
            4 => Operation::OpClear,
            _ => Operation::None,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Change<T> {
    operation: Operation,
    index: usize,
    item: T,
}

pub struct SyncList<T: Clone + Default + PartialEq + Eq + Sync + Send + 'static> {
    objects: Vec<T>,

    on_add: fn(usize),
    on_insert: fn(usize),
    on_set: fn(usize, &T),
    on_remove: fn(usize, &T),
    on_clear: fn(),
    on_change: fn(Operation, usize, &T),
    call_back: fn(Operation, usize, &T, &T),

    changes: Vec<Change<T>>,
    changes_ahead: usize,
}

impl<T: Clone + Default + PartialEq + Eq + Sync + Send + 'static> SyncList<T> {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
            on_add: |_: usize| {},
            on_insert: |_: usize| {},
            on_set: |_: usize, _: &T| {},
            on_remove: |_: usize, _: &T| {},
            on_clear: || {},
            on_change: |_: Operation, _: usize, _: &T| {},
            call_back: |_: Operation, _: usize, _: &T, _: &T| {},
            changes: Vec::new(),
            changes_ahead: 0,
        }
    }

    pub fn set_on_add(&mut self, on_add: fn(usize)) {
        self.on_add = on_add;
    }

    pub fn set_on_insert(&mut self, on_insert: fn(usize)) {
        self.on_insert = on_insert;
    }

    pub fn set_on_set(&mut self, on_set: fn(usize, &T)) {
        self.on_set = on_set;
    }

    pub fn set_on_remove(&mut self, on_remove: fn(usize, &T)) {
        self.on_remove = on_remove;
    }

    pub fn set_on_clear(&mut self, on_clear: fn()) {
        self.on_clear = on_clear;
    }

    pub fn set_on_change(&mut self, on_change: fn(Operation, usize, &T)) {
        self.on_change = on_change;
    }

    pub fn set_call_back(&mut self, call_back: fn(Operation, usize, &T, &T)) {
        self.call_back = call_back;
    }

    pub fn count(&self) -> usize {
        self.objects.len()
    }

    pub fn is_read_only(&self) -> bool {
        // TODO implement is_read_only
        false
    }

    fn add_operation(
        &mut self,
        operation: Operation,
        index: usize,
        old_value: &T,
        new_value: &T,
        check_access: bool,
    ) {
        if check_access && self.is_read_only() {
            // TODO log
            return;
        }

        let change = Change {
            operation,
            index,
            item: new_value.clone(),
        };

        // TODO implement IsRecording
        if true {
            self.changes.push(change);
        }

        match operation {
            Operation::OpAdd => {
                (self.on_add)(index);
                (self.on_change)(operation, index, new_value);
                (self.call_back)(operation, index, old_value, new_value);
            }
            Operation::OpSet => {
                (self.on_set)(index, new_value);
                (self.on_change)(operation, index, new_value);
                (self.call_back)(operation, index, old_value, new_value);
            }
            Operation::OpInsert => {
                (self.on_insert)(index);
                (self.on_change)(operation, index, new_value);
                (self.call_back)(operation, index, old_value, new_value);
            }
            Operation::OpRemoveAt => {
                (self.on_remove)(index, old_value);
                (self.on_change)(operation, index, new_value);
                (self.call_back)(operation, index, old_value, new_value);
            }
            Operation::OpClear => {
                (self.on_clear)();
                (self.on_change)(operation, index, new_value);
                (self.call_back)(operation, index, old_value, new_value);
            }
            _ => {}
        }
    }

    pub fn add(&mut self, item: &T) {
        self.objects.push(item.clone());
        self.add_operation(
            Operation::OpAdd,
            self.objects.len() - 1,
            &Default::default(),
            item,
            true,
        );
    }

    pub fn add_range(&mut self, items: &Vec<T>) {
        for item in items {
            self.add(item);
        }
    }

    pub fn clear(&mut self) {
        self.objects.clear();
        self.add_operation(
            Operation::OpClear,
            0,
            &Default::default(),
            &Default::default(),
            true,
        );
    }

    pub fn contains(&self, item: &T) -> bool {
        self.objects.contains(item)
    }

    pub fn copy_to(&self, array: &mut [T], array_index: usize) {
        for (i, item) in self.objects.iter().enumerate() {
            array[array_index + i] = item.clone();
        }
    }

    pub fn index_of(&self, item: &T) -> Option<usize> {
        self.objects.iter().position(|x| x == item)
    }

    pub fn find_index(&self, predicate: fn(&T) -> bool) -> Option<usize> {
        self.objects.iter().position(predicate)
    }

    pub fn insert(&mut self, index: usize, item: &T) {
        self.objects.insert(index, item.clone());
        self.add_operation(Operation::OpInsert, index, &Default::default(), item, true);
    }

    pub fn insert_range(&mut self, index: usize, items: &Vec<T>) {
        for (i, item) in items.iter().enumerate() {
            self.insert(index + i, item);
        }
    }

    pub fn remove(&mut self, item: &T) -> bool {
        match self.objects.iter().position(|x| x == item) {
            Some(index) => {
                self.objects.remove(index);
                self.add_operation(
                    Operation::OpRemoveAt,
                    index,
                    item,
                    &Default::default(),
                    true,
                );
                true
            }
            None => false,
        }
    }

    pub fn remove_at(&mut self, index: usize) {
        let item = self.objects.remove(index);
        self.add_operation(
            Operation::OpRemoveAt,
            index,
            &item,
            &Default::default(),
            true,
        );
    }

    pub fn remove_all(&mut self, predicate: fn(&T) -> bool) -> usize {
        let mut count = 0;
        let mut index = 0;
        while index < self.objects.len() {
            if predicate(&self.objects[index]) {
                self.remove_at(index);
                count += 1;
            } else {
                index += 1;
            }
        }
        count
    }
}

impl<T: Clone + Default + PartialEq + Eq + Sync + Send + 'static> SyncObject for SyncList<T> {
    fn sub_class_name() -> &'static str
    where
        Self: Sized,
    {
        todo!()
    }

    fn clear_changes(&mut self) {
        self.changes.clear();
    }

    fn on_serialize_all(&self, writer: &mut NetworkWriter) {
        writer.write_uint(self.objects.len() as u32);

        for item in self.objects.iter() {
            writer.write_blittable(item);
        }

        writer.write_uint(self.changes.len() as u32);
    }

    fn on_serialize_delta(&self, writer: &mut NetworkWriter) {
        writer.write_uint(self.changes.len() as u32);

        for change in &self.changes {
            writer.write_byte(change.operation as u8);
            match change.operation {
                Operation::OpAdd => {
                    writer.write_blittable(change.item.clone());
                }
                Operation::OpSet => {
                    writer.write_uint(change.index as u32);
                    writer.write_blittable(change.item.clone());
                }
                Operation::OpRemoveAt => {
                    writer.write_uint(change.index as u32);
                }
                Operation::OpClear => {}
                Operation::OpInsert => {}
                _ => {}
            }
        }
    }

    fn on_deserialize_all(&mut self, reader: &mut NetworkReader) {
        let count = reader.read_uint();

        self.objects.clear();
        self.changes.clear();

        for _ in 0..count {
            self.objects.push(reader.read_blittable());
        }

        self.changes_ahead = reader.read_uint() as usize;
    }

    fn on_deserialize_delta(&mut self, reader: &mut NetworkReader) {
        let changes_count = reader.read_uint() as usize;
        for _ in 0..changes_count {
            let operation = Operation::from_u8(reader.read_byte());
            let apply = self.changes_ahead == 0;
            let mut index = 0;
            let mut old_value = T::default();
            let mut new_value = T::default();
            match operation {
                Operation::OpAdd => {
                    new_value = reader.read_blittable();
                    if apply {
                        index = self.objects.len();
                        self.objects.push(new_value.clone());
                        self.add_operation(operation, index - 1, &old_value, &new_value, false);
                    }
                }
                Operation::OpSet => {}
                Operation::OpInsert => {
                    index = reader.read_uint() as usize;
                    new_value = reader.read_blittable();
                    if apply {
                        self.objects.insert(index, new_value.clone());
                        self.add_operation(operation, index, &old_value, &new_value, false);
                    }
                }
                Operation::OpRemoveAt => {
                    index = reader.read_uint() as usize;
                    if apply {
                        old_value = self.objects.remove(index);
                        self.add_operation(operation, index, &old_value, &new_value, false);
                    }
                }
                Operation::OpClear => {}
                Operation::None => {}
            }
            if !apply {
                self.changes_ahead -= 1;
            }
        }
    }

    fn reset(&mut self) {
        self.objects.clear();
        self.changes.clear();
        self.changes_ahead = 0;
    }
}
