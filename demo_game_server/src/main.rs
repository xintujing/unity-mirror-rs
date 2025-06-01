use unity_mirror_rs::mirror::NetworkManager;
use unity_mirror_rs::unity_engine::GameLooper;

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
    // let manager = NetworkRoomManager::default();
    // let arc = RevelArc::new(Box::new(manager));
    // let weak = arc.downgrade();
    // let action = Action::new(weak, NetworkRoomManager::qwer);
    //
    // println!("{}", action.call((5,)));
    //
    // return;

    NetworkManager::set_network_manager_prefab_path("Assets/Prefabs/NetworkRoomManager.prefab");
    NetworkManager::init();

    // NetworkServer.aaa();

    // NetworkManager::singleton::<NetworkRoomManager>(|network_room_manager| {
    //     network_room_manager.awake()
    // });

    // WorldManager::load_scene("Assets/Scenes/RoomScene.unity", LoadSceneMode::Single);
    //
    // let root_game_objects = WorldManager::root_game_objects();
    //
    // for root_game_object in root_game_objects.iter() {
    //     let game_object = root_game_object.get().unwrap();
    //     let weak_game_object = game_object.try_get_component::<NetworkBehaviour>().unwrap();
    //     let weak_network_transform_unreliable =
    //         weak_game_object.downcast::<NetworkTransformUnreliable>();
    //     let x = weak_network_transform_unreliable.unwrap();
    //     println!("qqqqqq {}", x.get().unwrap().buffer_reset_multiplier);
    // }
    // NetworkServer::listen()
    GameLooper::new().run();
}
