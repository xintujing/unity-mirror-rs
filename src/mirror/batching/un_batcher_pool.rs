#![allow(dead_code)]
use crate::mirror::batching::un_batcher::UnBatcher;
use crate::mirror::pool::Pool;
use lazy_static::lazy_static;
use std::sync::{Arc, Mutex};

lazy_static! {
    static ref UN_BATCHER_POOL: Arc<Mutex<Pool<UnBatcher>>> =
        Arc::new(Mutex::new(Pool::new(|| UnBatcher::new(), 10000)));
}

pub struct UnBatcherPool;

#[allow(unused)]
impl UnBatcherPool {
    pub fn count() -> usize {
        if let Ok(pool) = UN_BATCHER_POOL.lock() {
            pool.count()
        } else {
            println!("NetworkWriterPool::count() failed to lock NETWORK_WRITER_POOL");
            0
        }
    }

    pub fn get() -> UnBatcher {
        if let Ok(mut pool) = UN_BATCHER_POOL.lock() {
            let mut un_batcher = pool.get();
            un_batcher.clear();
            un_batcher
        } else {
            println!("NetworkWriterPool::get() failed to lock NETWORK_WRITER_POOL");
            UnBatcher::new()
        }
    }

    pub fn get_by_closure<T>(func: T)
    where
        T: FnOnce(&mut UnBatcher),
    {
        let mut un_batcher = Self::get();
        func(&mut un_batcher);
        Self::return_(un_batcher);
    }

    pub fn return_(mut un_batcher: UnBatcher) {
        if let Ok(mut pool) = UN_BATCHER_POOL.lock() {
            un_batcher.clear();
            pool.return_(un_batcher);
        } else {
            println!("NetworkWriterPool::return_() failed to lock NETWORK_WRITER_POOL");
        }
    }
}
