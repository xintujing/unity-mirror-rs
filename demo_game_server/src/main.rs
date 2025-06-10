#![allow(dead_code, unused)]

use unity_mirror_rs::mirror::NetworkManager;
use unity_mirror_rs::unity_engine::PlayerLooper;

use unity_mirror_rs::*;

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
                "[{}:{}] {} [{}] {} ",
                // 文件名和行号（使用 `unwrap_or` 处理空值）
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0),
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
