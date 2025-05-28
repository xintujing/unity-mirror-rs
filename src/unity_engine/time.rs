use once_cell::sync::Lazy;
use std::sync::atomic::Ordering::SeqCst;
use std::sync::atomic::{AtomicU16, AtomicU64};
use std::time::{Duration, Instant};

static mut START_INSTANT: Lazy<Option<Instant>> = Lazy::new(|| None); // 游戏开始时的时间
static mut FRAME_RATE: AtomicU16 = AtomicU16::new(60); // 60 FPS
static mut FRAME_COUNT: AtomicU64 = AtomicU64::new(0); // 帧计数器
static mut FIXED_DATA_TIME: AtomicU64 = AtomicU64::new(20); // 20 ms
static mut DEFAULT_PING_INTERVAL: f32 = 0.1; // 默认的ping间隔时间（单位：秒）

pub struct Time;

#[allow(static_mut_refs)]
#[allow(unused)]
impl Time {
    pub fn start_instant() {
        unsafe {
            if START_INSTANT.is_none() {
                *START_INSTANT = Some(Instant::now())
            } else {
                panic!("Time has already started");
            }
        }
    }

    pub fn unscaled_time() -> Instant {
        unsafe {
            match *START_INSTANT {
                None => {
                    panic!("Time has not started yet");
                }
                Some(start_instant) => start_instant,
            }
        }
    }
    pub fn unscaled_time_duration() -> Duration {
        Self::unscaled_time().elapsed()
    }

    pub fn unscaled_time_f64() -> f64 {
        Self::unscaled_time_duration().as_secs_f64()
    }

    pub fn frame_add() -> u64 {
        unsafe { FRAME_COUNT.fetch_add(1, SeqCst) }
    }

    pub fn get_frame_count() -> u64 {
        unsafe { FRAME_COUNT.load(SeqCst) }
    }

    pub fn get_frame_rate() -> f64 {
        unsafe { 1f64 / FRAME_RATE.load(SeqCst) as f64 }
    }

    pub fn get_frame_rate_duration() -> Duration {
        unsafe { Duration::from_secs_f64(1f64 / FRAME_RATE.load(SeqCst) as f64) }
    }

    pub fn set_frame_rate(frame_rate: u16) {
        unsafe {
            FRAME_RATE.store(frame_rate, SeqCst);
        }
    }

    pub fn get_fixed_data_time() -> u64 {
        unsafe { FIXED_DATA_TIME.load(SeqCst) }
    }

    pub fn get_fixed_data_time_duration() -> Duration {
        unsafe { Duration::from_millis(FIXED_DATA_TIME.load(SeqCst)) }
    }

    pub fn set_fixed_data_time(fixed_data_time_millis: u64) {
        unsafe {
            FIXED_DATA_TIME.store(fixed_data_time_millis, SeqCst);
        }
    }

    pub fn get_min_interval() -> Duration {
        unsafe {
            let fixed_data_time_millis = Self::get_fixed_data_time();
            let frame_rate_duration = Self::get_frame_rate_duration().as_millis() as u64;
            Duration::from_millis(u64::min(fixed_data_time_millis, frame_rate_duration))
        }
    }

    pub fn ping_interval() -> f64 {
        unsafe { DEFAULT_PING_INTERVAL as f64 }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct ExponentialMovingAverage {
    alpha: f64,
    initialized: bool,
    pub value: f64,
    pub variance: f64,
    pub standard_deviation: f64,
}

impl ExponentialMovingAverage {
    pub fn new(n: u32) -> Self {
        Self {
            alpha: 2.0 / (n as f64 + 1.0),
            value: 0.0,
            variance: 0.0,
            standard_deviation: 0.0,
            initialized: false,
        }
    }

    pub fn add(&mut self, new_value: f64) {
        if self.initialized {
            let delta = new_value - self.value;
            self.value += self.alpha * delta;
            self.variance = (1.0 - self.alpha) * (self.variance + self.alpha * delta.powi(2));
            self.standard_deviation = self.variance.sqrt();
        } else {
            self.value = new_value;
            self.initialized = true;
        }
    }

    pub fn reset(&mut self) {
        self.value = 0.0;
        self.variance = 0.0;
        self.standard_deviation = 0.0;
        self.initialized = false;
    }
}
