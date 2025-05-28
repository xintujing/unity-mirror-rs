#![allow(dead_code)]
use crate::mirror::batching::batcher::Batcher;
use crate::mirror::network_reader::NetworkReader;
use crate::mirror::network_writer::NetworkWriter;
use crate::mirror::network_writer_pool::NetworkWriterPool;
use std::collections::VecDeque;

pub struct UnBatcher {
    un_batches: VecDeque<NetworkWriter>,
    reader: NetworkReader,
    batch_timestamp: f64,
}

impl UnBatcher {
    pub fn new() -> UnBatcher {
        UnBatcher {
            un_batches: VecDeque::new(),
            reader: NetworkReader::new(Vec::new()),
            batch_timestamp: 0.0,
        }
    }

    pub fn batches_count(&self) -> usize {
        self.un_batches.len()
    }

    pub fn add_batch_with_array_segment(&mut self, data: &[u8]) -> bool {
        if data.len() < Batcher::TIMESTAMP_SIZE {
            return false;
        }
        let mut writer = NetworkWriterPool::get();
        writer.write_slice(data, 0, data.len());

        if self.un_batches.is_empty() {
            self.reader.set_slice(writer.to_slice());
            self.batch_timestamp = self.reader.read_blittable::<f64>();
        }
        self.un_batches.push_back(writer);
        true
    }

    pub fn add_batch_with_bytes(&mut self, data: &Vec<u8>) -> bool {
        if data.len() < Batcher::TIMESTAMP_SIZE {
            return false;
        }
        let mut writer = NetworkWriterPool::get();
        writer.write_slice(data, 0, data.len());

        if self.un_batches.is_empty() {
            self.reader.set_slice(writer.to_slice());
            self.batch_timestamp = self.reader.read_blittable::<f64>();
        }
        self.un_batches.push_back(writer);
        true
    }

    pub fn get_next_message(&mut self) -> Option<(&[u8], f64)> {
        let message: &[u8];
        let remote_time_stamp: f64;
        if self.un_batches.is_empty() {
            return None;
        }

        if self.reader.capacity() == 0 {
            return None;
        }

        if self.reader.remaining() == 0 {
            if let Some(write) = self.un_batches.pop_front() {
                NetworkWriterPool::return_(write);
            }

            if let Some(next) = self.un_batches.front() {
                self.reader.set_slice(next.to_slice());
                self.batch_timestamp = self.reader.read_blittable::<f64>();
            } else {
                return None;
            }
        }

        remote_time_stamp = self.batch_timestamp;

        if self.reader.remaining() == 0 {
            return None;
        }

        let size = self.reader.read_blittable_compress::<u64>() as usize;

        if self.reader.remaining() < size {
            return None;
        }

        message = self.reader.read_slice(size);

        Some((message, remote_time_stamp))
    }

    pub fn clear(&mut self) {
        self.un_batches.clear();
    }
}
