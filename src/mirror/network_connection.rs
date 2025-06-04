use crate::commons::revel_arc::RevelArc;
use crate::commons::revel_weak::RevelWeak;
use crate::mirror::batching::batcher::Batcher;
use crate::mirror::messages::message;
use crate::mirror::messages::message::MessageSerializer;
use crate::mirror::messages::network_ping_message::NetworkPingMessage;
use crate::mirror::network_writer_pool::NetworkWriterPool;
use crate::mirror::snapshot_interpolation::snapshot_interpolation::SnapshotInterpolation;
use crate::mirror::snapshot_interpolation::snapshot_interpolation_settings::SnapshotInterpolationSettings;
use crate::mirror::snapshot_interpolation::time_snapshot::TimeSnapshot;
use crate::mirror::transport::{TransportChannel, TransportManager};
use crate::mirror::{Authenticator, NetworkIdentity, NetworkServer, RemovePlayerOptions};
use crate::unity_engine::{ExponentialMovingAverage, Time};
use ordered_float::OrderedFloat;
use std::collections::{BTreeMap, HashMap, HashSet};

#[allow(unused)]
pub struct NetworkConnection {
    self_weak: RevelWeak<NetworkConnection>,

    // NetworkConnection
    pub id: u64,
    pub is_authenticated: bool,
    pub authentication_data: Option<Box<dyn Authenticator>>,
    pub is_ready: bool,
    pub last_message_time: f64,
    pub identity: RevelWeak<Box<NetworkIdentity>>,
    pub(crate) owned: Vec<RevelWeak<Box<NetworkIdentity>>>,
    pub remote_time_stamp: f64,
    batches: HashMap<TransportChannel, Batcher>,
    // NetworkConnectionToClient
    pub address: String,
    pub(crate) observing: Vec<RevelWeak<Box<NetworkIdentity>>>,
    drift_ema: ExponentialMovingAverage,
    delivery_time_ema: ExponentialMovingAverage,
    pub remote_timeline: f64,
    pub remote_timescale: f64,
    buffer_time_multiplier: f64,
    pub buffer_time: f64,
    pub snapshots: BTreeMap<OrderedFloat<f64>, TimeSnapshot>,
    pub snapshot_buffer_size_limit: i32,
    last_ping_time: f64,
    pub(crate) _rtt: ExponentialMovingAverage,
}

impl NetworkConnection {
    pub fn new(id: u64, address: String) -> RevelArc<Self> {
        let mut arc_connection = RevelArc::new(Self {
            self_weak: Default::default(),
            id,
            is_authenticated: false,
            authentication_data: None,
            is_ready: false,
            last_message_time: 0.0,
            identity: RevelWeak::default(),
            owned: Vec::new(),
            remote_time_stamp: 0.0,
            batches: HashMap::new(),
            address,
            observing: Vec::new(),
            drift_ema: ExponentialMovingAverage::new(
                NetworkServer.send_rate() as u32
                    * NetworkServer.client_snapshot_settings.drift_ema_duration as u32,
            ),
            delivery_time_ema: ExponentialMovingAverage::new(
                NetworkServer.send_rate() as u32
                    * NetworkServer
                        .client_snapshot_settings
                        .delivery_time_ema_duration as u32,
            ),
            remote_timeline: 0.0,
            remote_timescale: 0.0,
            buffer_time_multiplier: 2.0,
            buffer_time: 0.0,
            snapshots: BTreeMap::new(),
            snapshot_buffer_size_limit: 64,
            last_ping_time: 0.0,
            _rtt: ExponentialMovingAverage::new(Time::get_ping_window_size()),
        });

        arc_connection.buffer_time =
            NetworkServer.send_interval() * arc_connection.buffer_time_multiplier;
        arc_connection.self_weak = arc_connection.downgrade();

        arc_connection
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
                NetworkServer.send_interval(),
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
            NetworkServer.send_interval(),
            self.buffer_time,
            snapshot_settings.catchup_speed,
            snapshot_settings.slowdown_speed,
            &mut self.drift_ema,
            snapshot_settings.catchup_negative_threshold as f64,
            snapshot_settings.catchup_positive_threshold as f64,
            &mut self.delivery_time_ema,
        );
    }
    pub fn update_time_interpolation(&mut self) {
        if self.snapshots.len() >= self.snapshot_buffer_size_limit as usize {
            return;
        }
        // TODO
    }
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
        NetworkWriterPool::get_return(|writer| {
            message.serialize(writer);

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
                    if let Some(active) = TransportManager.active.get() {
                        active.server_send(self.id, writer.to_slice(), *channel);
                    }

                    writer.reset();
                }
            });
        }
    }
    pub fn disconnect(&mut self) {
        self.is_ready = false;
        if let Some(active) = TransportManager.active.get() {
            active.server_disconnect(self.id);
        }
    }

    // pub fn add_to_observing(&mut self, weak_identity: RevelWeak<Box<NetworkIdentity>>) {
    //     if let Some(identity) = weak_identity.get() {
    //         //TODO
    //     }
    //     self.observing.push(weak_identity);
    // }
    pub fn cleanup(&mut self) {
        // TODO
    }

    pub fn is_active(&self, timeout: f32) -> bool {
        Time::unscaled_time_f64() - self.last_message_time < timeout as f64
    }

    pub fn destroy_owned_objects(&mut self) {
        let mut temp = HashSet::new();
        for owned in self.owned.iter() {
            temp.insert(owned.clone());
        }

        for identity in temp.iter() {
            if let Some(identity) = identity.get() {
                if identity.scene_id != 0 {
                    NetworkServer::remove_player_for_connection(
                        self.self_weak.clone(),
                        RemovePlayerOptions::KeepActive,
                    );
                } else {
                    NetworkServer::destroy(identity.game_object.clone())
                }
            }
        }
    }
    pub fn remove_from_observings_observers(&mut self) {}

    pub fn remove_owned_object(&mut self, identity: RevelArc<Box<NetworkIdentity>>) {
        self.owned
            .retain(|owned| !owned.ptr_eq(&identity.downgrade()))
    }

    pub fn remove_from_observing(
        &mut self,
        identity: RevelArc<Box<NetworkIdentity>>,
        is_destroyed: bool,
    ) {
        self.observing
            .retain(|observing| !observing.ptr_eq(&identity.downgrade()));

        if !is_destroyed {
            if let Some(self_arc) = self.self_weak.upgrade() {
                NetworkServer::hide_for_connection(identity.clone(), self_arc)
            }
        }
    }
}
