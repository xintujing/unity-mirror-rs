use crate::commons::action::SelfMutAction;
use crate::commons::revel_arc::RevelArc;
use crate::commons::revel_weak::RevelWeak;
use crate::mirror::batching::un_batcher::UnBatcher;
use crate::mirror::messages::network_ping_message::NetworkPingMessage;
use crate::mirror::snapshot_interpolation::snapshot_interpolation::SnapshotInterpolation;
use crate::mirror::snapshot_interpolation::snapshot_interpolation_settings::SnapshotInterpolationSettings;
use crate::mirror::snapshot_interpolation::time_snapshot::TimeSnapshot;
use crate::mirror::transport::{TransportChannel, TransportManager};
use crate::mirror::NetworkTime;
use crate::mirror::NetworkWriter;
use crate::mirror::{NetworkConnection, NetworkIdentity, NetworkServer, RemovePlayerOptions};
use crate::unity_engine::{ExponentialMovingAverage, Time};
use ordered_float::OrderedFloat;
use std::collections::{BTreeMap, HashSet};
use unity_mirror_macro_rs::extends;

#[extends(NetworkConnection)]
#[derive(Default)]
pub struct NetworkConnectionToClient {
    pub self_weak: RevelWeak<Box<NetworkConnectionToClient>>,

    // *****************************
    /// 可靠的 RPC 缓冲区。
    pub reliable_rpc: RevelArc<NetworkWriter>,
    /// 不可靠的 RPC 缓冲区。
    pub unreliable_rpc: RevelArc<NetworkWriter>,
    /// 客户端的地址。
    pub address: String,
    /// 此连接可以观察到的网络对象。
    pub observing: HashSet<RevelWeak<Box<NetworkIdentity>>>,
    /// 用于处理网络消息的拆包操作。
    pub un_batcher: UnBatcher,
    /// 平均时间偏移，用于时间插值。
    drift_ema: ExponentialMovingAverage,
    /// 平均传输时间，用于时间插值。
    delivery_time_ema: ExponentialMovingAverage,
    /// 客户端的远程时间线。
    pub remote_timeline: f64,
    /// 客户端的远程时间缩放比例。
    pub remote_timescale: f64,
    /// 用于动态调整缓冲时间的倍数。
    buffer_time_multiplier: f64,
    /// 时间快照列表，用于时间插值。
    snapshots: BTreeMap<OrderedFloat<f64>, TimeSnapshot>,
    /// 时间快照缓冲区大小限制。
    pub snapshot_buffer_size_limit: usize,
    /// RTT的Ping（往返时间）
    last_ping_time: f64,
    /// 往返时间（以秒为单位），表示消息从服务器到客户端再返回服务器所需的时间。
    pub rtt: ExponentialMovingAverage,
}

impl NetworkConnectionToClient {
    pub fn buffer_time(&self) -> f64 {
        NetworkServer.send_interval() * self.buffer_time_multiplier
    }
    pub fn rtt(&self) -> f64 {
        self.rtt.value
    }
}

impl NetworkConnectionToClient {
    pub fn new(connection_id: u64, address: String) -> RevelArc<Box<Self>> {
        let mut connection_to_client = RevelArc::new(Box::new(Self::default()));
        connection_to_client.self_weak = connection_to_client.downgrade();

        connection_to_client.parent = NetworkConnection::new(connection_id);

        connection_to_client.parent.update = SelfMutAction::new(connection_to_client.self_weak.clone(), Self::update);
        connection_to_client.parent.send_to_transport = SelfMutAction::new(connection_to_client.self_weak.clone(), Self::send_to_transport);
        connection_to_client.parent.disconnect = SelfMutAction::new(connection_to_client.self_weak.clone(), Self::disconnect);

        connection_to_client.address = address;

        connection_to_client
    }
}

impl NetworkConnectionToClient {
    fn update(&mut self) {
        self.update_ping();
        self.parent.update_default();
    }

    fn send_to_transport(&mut self, segment: Vec<u8>, channel_id: TransportChannel) {
        TransportManager.active.server_send(
            self.connection_id,
            &segment,
            channel_id,
        )
    }

    pub fn disconnect(&mut self) {
        self.is_ready = false;
        self.reliable_rpc.reset();
        self.unreliable_rpc.reset();
        TransportManager
            .active
            .server_disconnect(self.connection_id)
    }
}

impl NetworkConnectionToClient {
    pub fn on_time_snapshot(&mut self, snapshot: TimeSnapshot) {
        if self.snapshots.len() > self.snapshot_buffer_size_limit {
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

        let buffer_time = self.buffer_time();
        SnapshotInterpolation::insert_and_adjust(
            &mut self.snapshots,
            self.snapshot_buffer_size_limit as usize,
            snapshot,
            &mut self.remote_timeline,
            &mut self.remote_timescale,
            NetworkServer.send_interval(),
            buffer_time,
            snapshot_settings.catchup_speed,
            snapshot_settings.slowdown_speed,
            &mut self.drift_ema,
            snapshot_settings.catchup_negative_threshold as f64,
            snapshot_settings.catchup_positive_threshold as f64,
            &mut self.delivery_time_ema,
        );
    }
    pub fn update_time_interpolation(&mut self) {
        if self.snapshots.len() > 0 {
            SnapshotInterpolation::step_time(
                Time::unscaled_time_f64(),
                &mut self.remote_timeline,
                self.remote_timescale,
            );

            SnapshotInterpolation::step_interpolation(&mut self.snapshots, self.remote_timescale);
        }
    }
    fn update_ping(&mut self) {
        if NetworkTime.local_time() >= self.last_ping_time + NetworkTime.ping_interval as f64 {
            let message = NetworkPingMessage::new(NetworkTime.local_time(), 0f64);
            self.send_message(message, TransportChannel::Reliable);
            self.last_ping_time = NetworkTime.local_time();
        }
    }
    pub fn add_to_observing(&mut self, identity: RevelArc<Box<NetworkIdentity>>) {
        self.observing.insert(identity.downgrade());

        if let Some(self_arc) = self.self_weak.upgrade() {
            NetworkServer::show_for_connection(identity, self_arc);
        }
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
    pub fn remove_from_observings_observers(&mut self) {
        for identity in self.observing.iter() {
            if let (Some(self_arc), Some(mut identity)) = (self.self_weak.upgrade(), identity.upgrade()) {
                identity.remove_observer(self_arc)
            }
        }
        self.observing.clear();
    }

    pub fn add_owned_object(&mut self, identity: RevelArc<Box<NetworkIdentity>>) {
        self.owned.insert(identity);
    }
    pub fn remove_owned_object(&mut self, identity: RevelArc<Box<NetworkIdentity>>) {
        self.owned.retain(|owned| !owned.ptr_eq(&identity.clone()))
    }
    pub fn destroy_owned_objects(&mut self) {
        let mut temp = HashSet::new();
        for owned in self.owned.iter() {
            temp.insert(owned.clone());
        }

        for identity in temp.iter() {
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
