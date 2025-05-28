use crate::commons::revel_weak::RevelWeak;
use crate::mirror::authenticator::authenticator::Authenticator;
use crate::mirror::authenticator::basic_authenticator::AuthResponseMessage;
use crate::mirror::batching::batcher::Batcher;
use crate::mirror::batching::un_batcher::UnBatcher;
use crate::mirror::transport::TransportChannel;
use crate::mirror::NetworkIdentity;
use std::collections::{BTreeMap, HashMap, HashSet};
use crate::mirror::snapshot_interpolation::time_snapshot::TimeSnapshot;
use crate::unity_engine::ExponentialMovingAverage;

pub struct NetworkConnection {
    // NetworkConnection
    pub id: u64,
    pub is_authenticated: bool,
    pub authentication_data: Option<Box<dyn Authenticator>>,
    pub is_ready: bool,
    pub last_message_time: f64,
    pub identity: RevelWeak<Box<NetworkIdentity>>,
    pub owned: HashSet<RevelWeak<Box<NetworkIdentity>>>,
    pub remote_time_stamp: f64,
    // Batcher,
    batches: HashMap<TransportChannel, Batcher>,
    // NetworkConnectionToClient
    pub address: String,
    pub observing: HashSet<RevelWeak<Box<NetworkIdentity>>>,
    pub un_batcher: UnBatcher,
    drift_ema: ExponentialMovingAverage,
    delivery_time_ema: ExponentialMovingAverage,
    pub remote_timeline: f64,
    pub remote_timescale: f64,
    buffer_time_multiplier: f64,
    pub buffer_time: f64,
    pub snapshots: BTreeMap<f64, TimeSnapshot>,
    pub snapshot_buffer_size_limit: i32,
    last_ping_time: f64,
}

impl NetworkConnection {
    pub(crate) fn send_message(
        &self,
        auth_response_message: &mut AuthResponseMessage,
        transport_channel: TransportChannel,
    ) {
        todo!()
    }
}

impl NetworkConnection {
    pub(crate) fn disconnect(&self) {
        todo!()
    }
}
