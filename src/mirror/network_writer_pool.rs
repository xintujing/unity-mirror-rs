use crate::mirror::pool::Pool;
use crate::mirror::NetworkWriter;
use lazy_static::lazy_static;
use std::sync::{Arc, Mutex};

lazy_static! {
    static ref NETWORK_WRITER_POOL: Arc<Mutex<Pool<NetworkWriter>>> =
        Arc::new(Mutex::new(Pool::new(|| NetworkWriter::new(), 1000)));
}

pub struct NetworkWriterPool;

#[allow(unused)]
impl NetworkWriterPool {
    pub fn count() -> usize {
        if let Ok(pool) = NETWORK_WRITER_POOL.lock() {
            pool.count()
        } else {
            println!("NetworkWriterPool::count() failed to lock NETWORK_WRITER_POOL");
            0
        }
    }

    pub fn get() -> NetworkWriter {
        if let Ok(mut pool) = NETWORK_WRITER_POOL.lock() {
            let mut writer = pool.get();
            writer.reset();
            writer
        } else {
            println!("NetworkWriterPool::get() failed to lock NETWORK_WRITER_POOL");
            NetworkWriter::new()
        }
    }

    pub fn get_by_closure<T>(func: T)
    where
        T: FnOnce(&mut NetworkWriter),
    {
        let mut writer = Self::get();
        func(&mut writer);
        Self::return_(writer);
    }

    pub fn return_(mut writer: NetworkWriter) {
        if let Ok(mut pool) = NETWORK_WRITER_POOL.lock() {
            writer.reset();
            pool.return_(writer);
        } else {
            println!("NetworkWriterPool::return_() failed to lock NETWORK_WRITER_POOL");
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_network_writer_pool() {}
}
