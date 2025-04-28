use crate::mirror::network_reader::NetworkReader;
use crate::mirror::network_writer::NetworkWriter;
use std::fmt::Debug;

pub trait SyncObject: Debug {
    fn set_on_dirty(&mut self, x: Box<dyn Fn() -> u64>);
    fn set_is_writable(&mut self, x: Box<dyn Fn() -> bool>);
    fn set_is_recording(&mut self, x: Box<dyn Fn() -> bool>);
    fn on_serialize_all(&self, writer: &mut NetworkWriter);
    fn on_serialize_delta(&self, writer: &mut NetworkWriter);
    fn on_deserialize_all(&mut self, reader: &mut NetworkReader);
    fn on_deserialize_delta(&mut self, reader: &mut NetworkReader);
}
