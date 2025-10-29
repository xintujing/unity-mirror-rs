mod authenticator;
mod metadata_settings;
mod poker_helper;
mod scripts;
mod util_syc;

use unity_mirror_rs::mirror::NetworkManager;
use unity_mirror_rs::unity_engine::PlayerLooper;

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
    // 启动内存分析
    // let guard = pprof::ProfilerGuard::new(100).unwrap(); // 每100ms采样一次
    //
    // { Logic }
    //
    // if let Ok(report) = guard.report().build() {
    //     let file = std::fs::File::create("flamegraph.svg").unwrap();
    //     report.flamegraph(file).unwrap();
    // }

    PlayerLooper::init();
    NetworkManager::init("Assets/24打大A2D/Prefabs/NetworkManagerExt.prefab");
    PlayerLooper::run();
}
