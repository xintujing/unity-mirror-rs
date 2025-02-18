use std::sync::Arc;

pub struct NetworkConnection {
    conn_id: Arc<parking_lot::RwLock<u64>>,
    net_id: Arc<parking_lot::RwLock<u32>>,
    owned: Vec<u32>,
    // reliable_batcher: Batcher,
    // unreliable_batcher: Batcher,
    is_ready: bool,
    last_message_time: f64,
    last_ping_time: f64,
    remote_time_stamp: f64,
    first_conn_loc_time_stamp: f64,
    is_authenticated: bool,
    // authentication_data: Option<Box<RwLock<dyn NetworkMessageTrait>>>,
}
