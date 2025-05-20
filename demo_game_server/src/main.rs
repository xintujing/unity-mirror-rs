use unity_mirror_rs::unity_engine::mirror::components::network_transform::network_transform_base::NetworkTransformBase;
use unity_mirror_rs::unity_engine::mirror::components::network_transform::network_transform_unreliable::NetworkTransformUnreliable;
use unity_mirror_rs::unity_engine::{LoadSceneMode, WorldManager};

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
    WorldManager::load_scene("Assets/Scenes/RoomScene.unity", LoadSceneMode::Single);

    let vec = WorldManager::root_game_objects();
    vec.iter().for_each(|weak_game_object| {
        if let Some(game_object) = weak_game_object.get() {
            println!("{}", game_object.name);
            if let Some(weak_network_identity) =
                game_object.try_get_component::<NetworkTransformBase>()
            {
                let weak = weak_network_identity.to::<NetworkTransformUnreliable>();
                println!("aaa {}", weak.get().unwrap().scale_sensitivity);
            }
        }
    });

    // GameLooper::new().run();
}
