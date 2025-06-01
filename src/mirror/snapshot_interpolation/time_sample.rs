use std::time::Instant;

#[derive(Copy, Clone)]
pub struct TimeSample {
    // 记录开始时间
    begin_time: Instant,

    // 指数移动平均值
    ema: ExponentialMovingAverage,

    // 平均值
    average: f64,
}

#[allow(unused)]
impl TimeSample {
    // const PRECISION_FACTOR: f64 = 1_000_000.0; // 1 million for microseconds precision

    // 新建一个 TimeSample
    pub fn new(n: u32) -> Self {
        Self {
            begin_time: Instant::now(),
            ema: ExponentialMovingAverage::new(n),
            average: 0.0,
        }
    }

    // 开始计时
    pub fn begin(&mut self) {
        self.begin_time = Instant::now();
    }

    // 结束计时
    pub fn end(&mut self) {
        // Add duration in seconds to accumulated durations
        let elapsed = self.begin_time.elapsed().as_secs_f64();
        self.ema.add(elapsed);

        // Expose new average thread safely
        self.average = self.ema.value();
    }

    // 获取平均值
    pub fn average(&self) -> f64 {
        self.average
    }
}

#[derive(Copy, Clone)]
struct ExponentialMovingAverage {
    n: u32,
    value: f64,
}

#[allow(unused)]
impl ExponentialMovingAverage {
    fn new(n: u32) -> Self {
        Self { n, value: 0.0 }
    }

    fn add(&mut self, sample: f64) {
        let alpha = 2.0 / (self.n as f64 + 1.0);
        self.value = alpha * sample + (1.0 - alpha) * self.value;
    }

    fn value(&self) -> f64 {
        self.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_time_sample() {
        let mut ts = TimeSample::new(10);

        // Simulate multiple begin/end calls with varying durations
        for _ in 0..20 {
            ts.begin();
            thread::sleep(Duration::from_millis(100));
            ts.end();
        }

        // The average should be close to 0.1 seconds (100 milliseconds)
        let average = ts.average();
        assert!(
            average > 0.09 && average < 0.11,
            "Average time is not within expected range: {}",
            average
        );

        // Simulate a longer duration
        ts.begin();
        thread::sleep(Duration::from_secs(2));
        ts.end();

        // The average should now be closer to 0.1 seconds due to EMA
        let average = ts.average();
        println!("Average time: {}", average);
    }
}
