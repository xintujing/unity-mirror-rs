use crate::commons::RevelArc;
use crate::mirror::messages::network_ping_message::NetworkPingMessage;
use crate::mirror::messages::network_pong_message::NetworkPongMessage;
use crate::mirror::transport::TransportChannel;
use crate::mirror::transport::TransportChannel::Reliable;
use crate::mirror::{NetworkConnectionToClient, NetworkServer};
use crate::unity_engine::{ExponentialMovingAverage, Time};
use once_cell::sync::Lazy;
use std::ops::{Deref, DerefMut};

const DEFAULT_PING_INTERVAL: f32 = 0.1;
pub const PING_WINDOW_SIZE: i32 = 50;

static mut NETWORK_TIME_STATIC: Lazy<NetworkTimeStatic> = Lazy::new(|| NetworkTimeStatic {
    ping_interval: DEFAULT_PING_INTERVAL,
    last_ping_time: 0.0,
    rtt: ExponentialMovingAverage::new(PING_WINDOW_SIZE as u32),
    // local_time: 0.0,
    prediction_error_window_size: 20,
    prediction_error_unadjusted: ExponentialMovingAverage::new(20),
    prediction_error_adjusted: 0.0,
    predicted_time: 0.0,
});

#[allow(unused)]
pub struct NetworkTimeStatic {
    pub ping_interval: f32,

    last_ping_time: f64,
    rtt: ExponentialMovingAverage,

    // local_time: f64,
    prediction_error_window_size: i32,
    prediction_error_unadjusted: ExponentialMovingAverage,
    prediction_error_adjusted: f64,
    predicted_time: f64,
}

#[allow(unused)]
impl NetworkTimeStatic {
    pub fn local_time(&self) -> f64 {
        Time::unscaled_time_f64()
    }
    pub fn time(&self) -> f64 {
        self.local_time()
    }

    pub fn prediction_error_unadjusted(&self) -> f64 {
        self.prediction_error_unadjusted.value
    }

    pub fn prediction_error_adjusted(&self) -> f64 {
        self.prediction_error_adjusted
    }

    pub fn get_predicted_time(&self) -> f64 {
        if NetworkServer.active {
            self.local_time()
        } else {
            self.local_time() + self.prediction_error_adjusted()
        }
    }

    pub fn offset(&self) -> f64 {
        self.local_time() - self.time()
    }

    pub fn rtt(&self) -> f64 {
        self.rtt.value
    }

    pub fn rtt_variance(&self) -> f64 {
        self.rtt.variance
    }
}

pub struct NetworkTime;

impl Deref for NetworkTime {
    type Target = NetworkTimeStatic;

    fn deref(&self) -> &Self::Target {
        #[allow(static_mut_refs)]
        unsafe {
            &NETWORK_TIME_STATIC
        }
    }
}

impl DerefMut for NetworkTime {
    fn deref_mut(&mut self) -> &mut Self::Target {
        #[allow(static_mut_refs)]
        unsafe {
            &mut NETWORK_TIME_STATIC
        }
    }
}

impl NetworkTime {
    pub fn on_server_ping(
        &mut self,
        mut connection: RevelArc<Box<NetworkConnectionToClient>>,
        message: NetworkPingMessage,
        _: TransportChannel,
    ) {
        let unadjusted_error = self.local_time() - message.local_time;
        let adjusted_error = self.local_time() - message.predicted_time_adjusted;

        let pong_message =
            NetworkPongMessage::new(message.local_time, unadjusted_error, adjusted_error);
        connection.send_message(pong_message, Reliable);
    }

    pub fn on_server_pong(
        &mut self,
        mut connection: RevelArc<Box<NetworkConnectionToClient>>,
        message: NetworkPongMessage,
        _: TransportChannel,
    ) {
        if message.local_time > self.local_time() {
            return;
        }

        let new_rtt = self.local_time() - message.local_time;
        connection.rtt.add(new_rtt);
    }

    pub fn reset_statics(&mut self) {
        self.ping_interval = DEFAULT_PING_INTERVAL;
        self.last_ping_time = 0.0;
        self.rtt = ExponentialMovingAverage::new(PING_WINDOW_SIZE as u32);
    }
}
