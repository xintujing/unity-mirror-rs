use crate::commons::revel_weak::RevelWeak;
use crate::mirror::authenticator::authenticator::Authenticator;
use crate::mirror::batching::batcher::Batcher;
use crate::mirror::batching::un_batcher::UnBatcher;
use crate::mirror::messages::message;
use crate::mirror::messages::message::MessageSerializer;
use crate::mirror::messages::network_ping_message::NetworkPingMessage;
use crate::mirror::network_writer_pool::NetworkWriterPool;
use crate::mirror::snapshot_interpolation::snapshot_interpolation::SnapshotInterpolation;
use crate::mirror::snapshot_interpolation::snapshot_interpolation_settings::SnapshotInterpolationSettings;
use crate::mirror::snapshot_interpolation::time_snapshot::TimeSnapshot;
use crate::mirror::transport::{transport_manager, TransportChannel};
use crate::mirror::NetworkIdentity;
use crate::unity_engine::{ExponentialMovingAverage, Time};
use ordered_float::OrderedFloat;
use std::collections::{BTreeMap, HashMap, HashSet};

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
    pub snapshots: BTreeMap<OrderedFloat<f64>, TimeSnapshot>,
    pub snapshot_buffer_size_limit: i32,
    last_ping_time: f64,
}

impl NetworkConnection {
    pub fn new(id: u64, address: String) -> Self {
        Self {
            id,
            is_authenticated: false,
            authentication_data: None,
            is_ready: false,
            last_message_time: 0.0,
            identity: RevelWeak::default(),
            owned: HashSet::new(),
            remote_time_stamp: 0.0,
            batches: HashMap::new(),
            address,
            observing: HashSet::new(),
            un_batcher: UnBatcher::new(),
            // TODO
            drift_ema: ExponentialMovingAverage::new(1),
            // TODO
            delivery_time_ema: ExponentialMovingAverage::new(1),
            remote_timeline: 0.0,
            remote_timescale: 0.0,
            buffer_time_multiplier: 2.0,
            // TODO
            buffer_time: 0.0,
            snapshots: BTreeMap::new(),
            snapshot_buffer_size_limit: 64,
            last_ping_time: 0.0,
        }
    }
    pub fn update(&mut self) {
        self.update_ping();
        self.send_batches()
    }
    pub fn on_time_snapshot(&mut self, snapshot: TimeSnapshot) {
        if self.snapshots.len() >= self.snapshot_buffer_size_limit as usize {
            return;
        }

        let snapshot_settings = SnapshotInterpolationSettings::new();

        // dynamic adjustment
        if snapshot_settings.dynamic_adjustment {
            self.buffer_time_multiplier = SnapshotInterpolation::dynamic_adjustment(
                // TODO NetworkServer.send_interval as f64
                0.0,
                self.delivery_time_ema.standard_deviation,
                snapshot_settings.dynamic_adjustment_tolerance as f64,
            )
        }

        SnapshotInterpolation::insert_and_adjust(
            &mut self.snapshots,
            self.snapshot_buffer_size_limit as usize,
            snapshot,
            &mut self.remote_timeline,
            &mut self.remote_timescale,
            // TODO NetworkServer.send_interval as f64
            0.0,
            self.buffer_time,
            snapshot_settings.catchup_speed,
            snapshot_settings.slowdown_speed,
            &mut self.drift_ema,
            snapshot_settings.catchup_negative_threshold as f64,
            snapshot_settings.catchup_positive_threshold as f64,
            &mut self.delivery_time_ema,
        );
    }
    pub fn update_time_interpolation(&mut self) {}
    pub fn send(&mut self, segment: &[u8], channel: TransportChannel) {
        if !self.batches.contains_key(&channel) {
            self.batches.insert(channel, Batcher::new(1500));
        };

        self.batches
            .get_mut(&channel)
            .unwrap()
            .add_message(segment, Time::unscaled_time_f64());
    }
    pub fn send_message<T>(&mut self, message: &mut T, channel: TransportChannel)
    where
        T: MessageSerializer,
    {
        NetworkWriterPool::get_return(|mut writer| {
            message.serialize(&mut writer);

            let max_size = message::max_message_size(channel);

            if writer.position > max_size {
                return;
            }

            self.send(writer.to_slice(), channel);
        });
    }
    fn update_ping(&mut self) {
        let local_time = Time::unscaled_time_f64();
        if local_time >= self.last_ping_time + Time::ping_interval() {
            self.last_ping_time = local_time;
            self.send_message(
                &mut NetworkPingMessage::new(local_time, 0.0),
                TransportChannel::Unreliable,
            );
        }
    }
    fn send_batches(&mut self) {
        for (channel, batcher) in self.batches.iter_mut() {
            NetworkWriterPool::get_return(|writer| {
                while batcher.get_batcher_writer(writer) {
                    if let Some(transport) = transport_manager() {
                        transport.server_send(self.id, writer.to_slice(), *channel);
                    }
                    writer.reset();
                }
            });
        }
    }
    pub fn disconnect(&mut self) {
        self.is_ready = false;
        if let Some(transport) = transport_manager() {
            transport.server_disconnect(self.id);
        }
    }
}
