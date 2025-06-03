#![allow(dead_code)]
use crate::mirror::network_writer::NetworkWriter;
use crate::mirror::network_writer_pool::NetworkWriterPool;
use std::collections::VecDeque;
use crate::mirror::compress::Compress;

pub struct Batcher {
    threshold: usize,
    batches: VecDeque<NetworkWriter>,
    batcher: Option<NetworkWriter>,
    batch_timestamp: f64,
}

impl Batcher {
    pub const TIMESTAMP_SIZE: usize = size_of::<f64>();

    pub fn new(threshold: usize) -> Self {
        Self {
            threshold,
            batches: VecDeque::new(),
            batcher: None,
            batch_timestamp: 0.0,
        }
    }

    pub fn message_header_size(message_size: usize) -> usize {
        Compress.var_uint_size(message_size as u64)
    }

    pub fn max_message_overhead(message_size: usize) -> usize {
        Self::TIMESTAMP_SIZE + Self::message_header_size(message_size)
    }

    pub fn add_message(&mut self, message: &[u8], timestamp: f64) {
        if self.batcher.is_some() && self.batch_timestamp != timestamp {
            if let Some(batcher) = self.batcher.take() {
                self.batch_timestamp = 0.0;
                self.batches.push_back(batcher);
            }
        }

        let header_size = Compress.var_uint_size(message.len() as u64);
        let needed_size = header_size + message.len();

        if let Some(ref batcher) = self.batcher {
            if batcher.position + needed_size > self.threshold {
                if let Some(batcher) = self.batcher.take() {
                    self.batch_timestamp = 0.0;
                    self.batches.push_back(batcher);
                }
            }
        }

        if self.batcher.is_none() {
            self.batch_timestamp = timestamp;
            let mut batcher = NetworkWriterPool::get();
            batcher.write_blittable(self.batch_timestamp);
            self.batcher = Some(batcher);
        }

        if let Some(ref mut batcher) = self.batcher {
            batcher.write_blittable_compress(message.len() as u64);
            batcher.write_slice(message, 0, message.len());
        }
    }

    pub fn get_batcher_writer(&mut self, writer: &mut NetworkWriter) -> bool {
        if let Some(batcher) = self.batches.pop_front() {
            Self::copy_and_return_batcher(batcher, writer);
            return true;
        }
        if let Some(batcher) = self.batcher.take() {
            Self::copy_and_return_batcher(batcher, writer);
            return true;
        }
        false
    }

    fn copy_and_return_batcher(batcher: NetworkWriter, writer: &mut NetworkWriter) {
        if writer.position != 0 {
            log::error!("Writer must be empty");
            writer.reset();
        }
        let segment = batcher.to_slice();
        writer.write_slice(segment, 0, segment.len());
        NetworkWriterPool::return_(batcher);
    }

    pub fn clear(&mut self) {
        if let Some(batcher) = self.batcher.take() {
            NetworkWriterPool::return_(batcher);
        }
        for batcher in self.batches.drain(..) {
            NetworkWriterPool::return_(batcher);
        }
    }
}
