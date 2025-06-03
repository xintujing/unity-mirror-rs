use crate::mirror::NetworkLoop;
use crate::unity_engine::time::Time;
use crate::unity_engine::world::WorldManager;
use std::time::Instant;

pub struct PlayerLooper {
    last_frame_time: Instant,
    last_fixed_time: Instant,
}

impl PlayerLooper {
    pub fn run() {
        PlayerLooper {
            last_frame_time: Instant::now(),
            last_fixed_time: Instant::now(),
        }
        ._run()
    }

    pub fn fixed_update(&mut self) {
        if let Some(elapsed) = self
            .last_fixed_time
            .checked_sub(Time::unscaled_time().elapsed())
        {
            if elapsed.elapsed() >= Time::get_fixed_data_time_duration() {
                self.last_fixed_time = Instant::now();
                WorldManager::fixed_update();
            }
        }
    }

    pub fn frame_update(&mut self) {
        if let Some(elapsed) = self
            .last_frame_time
            .checked_sub(Time::unscaled_time().elapsed())
        {
            if elapsed.elapsed() >= Time::get_frame_rate_duration() {
                self.last_frame_time = Instant::now();
                NetworkLoop.network_early_update(); // TODO 补充注册逻辑
                WorldManager::update();
                WorldManager::late_update();
                NetworkLoop.network_late_update(); // TODO 补充注册逻辑
            }
        }
    }

    fn _run(&mut self) {
        Time::start_instant();
        loop {
            let tmp_instant = Instant::now();
            self.fixed_update();
            self.frame_update();

            let elapsed = tmp_instant.elapsed();
            if elapsed < Time::get_min_interval() {
                let diff_duration = Time::get_min_interval().abs_diff(elapsed);
                std::thread::sleep(diff_duration);
            }
        }
    }
}
