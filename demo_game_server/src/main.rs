#![allow(dead_code, unused)]

use crate::mirror::{DataTypeSerializer, NetworkManager, NetworkWriter};
use crate::unity_engine::PlayerLooper;

use unity_mirror_rs::*;
#[macro_use]
extern crate unity_mirror_rs;

mod backend_metadata;
mod scripts;

#[ctor::ctor]
fn init_logger() {
    use colored::Colorize;
    use log::Level;
    use std::io::Write;
    env_logger::Builder::new()
        .format_level(true)
        .filter_level(log::LevelFilter::Debug)
        .format(|buf, record| {
            writeln!(
                buf,
                "[{}] {} [{}] {} ",
                format!(
                    "\x1B]8;;{}\x1B\\{}\x1B]8;;\x1B\\",
                    format!(
                        "{}:{}",
                        record.file_static().unwrap_or_default(),
                        record.line().unwrap_or(0),
                    ),
                    record.module_path_static().unwrap_or_default(),
                ),
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                // 使用自定义颜色显示日志级别
                match record.level() {
                    Level::Error => "ERROR".red().to_string(),
                    Level::Warn => "WARN".yellow().to_string(),
                    Level::Info => "INFO".green().to_string(),
                    Level::Debug => "DEBUG".blue().to_string(),
                    Level::Trace => "TRACE".purple().to_string(),
                },
                // 日志内容
                record.args(),
            )
        })
        .init();
}

fn main() {
    NetworkManager::init("Assets/Prefabs/NetworkRoomManager.prefab");
    PlayerLooper::run();
}
