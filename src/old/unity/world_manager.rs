use crate::commons::reference::Reference;
use crate::metadata_settings::metadata::Metadata;
use crate::metadata_settings::unity::metadata_prefab::MetadataPrefab;
use crate::unity::{GameObject, Transform};
use once_cell::sync::Lazy;
use std::cell::RefCell;
use std::sync::{Arc, RwLock};

static mut WORLD_MANAGER_STATE: Lazy<Arc<RwLock<WorldManagerState>>> =
    Lazy::new(|| Default::default());

#[derive(Default)]
pub struct WorldManagerState {
    pub scene_path: String,
    pub game_objects: Vec<Arc<RwLock<GameObject>>>,
}
pub struct WorldManager;

impl WorldManager {
    pub fn get_mut() -> Option<std::sync::RwLockWriteGuard<'static, WorldManagerState>> {
        unsafe {
            if let Ok(state) = WORLD_MANAGER_STATE.write() {
                return Some(state);
            }
            None
        }
    }

    pub fn get() -> Option<std::sync::RwLockReadGuard<'static, WorldManagerState>> {
        unsafe {
            if let Ok(state) = WORLD_MANAGER_STATE.read() {
                return Some(state);
            }
            None
        }
    }

    pub fn change_scene(scene_path: &str) -> bool {
        if let Some(metadata_scene) = Metadata::get_scene(scene_path) {
            let mut world = WorldManagerState {
                scene_path: scene_path.to_string(),
                game_objects: vec![],
            };

            for (_, metadata_prefab) in metadata_scene.iter() {
                let arc_game_object = Self::metadata_prefab_instance(None, metadata_prefab);
                Self::recursive_children(
                    Reference::new(Arc::downgrade(&arc_game_object)),
                    &metadata_prefab.children,
                );
                world.game_objects.push(arc_game_object)
            }

            unsafe { *WORLD_MANAGER_STATE = Arc::new(RwLock::new(world)) }

            true
        } else {
            false
        }
    }

    fn metadata_prefab_instance(
        parent: Option<Reference<GameObject>>,
        metadata_prefab: &MetadataPrefab,
    ) -> Arc<RwLock<GameObject>> {
        let mut game_object = GameObject {
            id: "".to_string(),
            name: metadata_prefab.name.clone(),
            tag: metadata_prefab.tag.clone(),
            layer: metadata_prefab.layer,
            is_static: metadata_prefab.is_static,
            is_active: metadata_prefab.is_active,
            asset_id: metadata_prefab.asset_id,
            asset_path: metadata_prefab.asset_path.clone(),
            parent: parent.clone(),
            components: Default::default(),
            transform: Arc::default(),
            children: vec![],
        };

        let mut transform = Transform {
            parent: None,
            children: vec![],
            game_object: Default::default(),
            position: Default::default(),
            local_position: Default::default(),
            rotation: Default::default(),
            local_rotation: Default::default(),
            local_scale: Default::default(),
        };

        for (full_name,settings) in metadata_prefab.components.iter() {

        }


        Self::load_component();

        let mut parent_transform = RefCell::new(None);
        if let Some(parent) = &parent {
            parent.get(|parent| {
                *parent_transform.borrow_mut() =
                    Some(Reference::new(Arc::downgrade(&parent.transform)))
            })
        }
        // let option = *parent_transform.borrow_mut();
        // let weak = Arc::downgrade(&(*));
        transform.parent = parent_transform.borrow_mut().clone();

        let arc_transform = Arc::new(RwLock::new(transform));
        game_object.transform = arc_transform;
        Arc::new(RwLock::new(game_object))
    }

    fn recursive_children(parent: Reference<GameObject>, children: &Vec<MetadataPrefab>) {
        for metadata_prefab in children {
            // parent.get(|parent| {
            //     println!(
            //         "parent.name: {} current.name: {}",
            //         parent.name, metadata_prefab.name
            //     )
            // });

            // 生成预置体结构体
            let arc_game_object =
                Self::metadata_prefab_instance(Some(parent.clone()), metadata_prefab);
            if let Ok(rw_game_object) = arc_game_object.read() {
                // println!("{}", rw_game_object.transform.is_none());
            }
            let weak_game_object = Arc::downgrade(&arc_game_object);
            parent.get_mut(|mut mut_parent| mut_parent.children.push(arc_game_object.clone()));

            if metadata_prefab.children.len() > 0 {
                Self::recursive_children(
                    Reference::new(weak_game_object.clone()),
                    &metadata_prefab.children,
                );
            }

            if let Some(arc_game_object) = weak_game_object.upgrade() {
                if let Ok(game_object) = arc_game_object.write() {
                    if let Ok(mut transform) = game_object.transform.write() {
                        for child_game_object in game_object.children.iter() {
                            if let Ok(r_child_game_object) = child_game_object.read() {
                                transform.children.push(Reference::new(Arc::downgrade(
                                    &r_child_game_object.transform,
                                )))
                            }
                        }
                    }
                } else {
                    println!("arc_game_object.write() failed")
                };
            }
        }
    }
    fn load_component() {}

    pub fn current_scene_path() -> &'static str {
        if let Some(state) = Self::get() {
            Box::leak(state.scene_path.clone().into_boxed_str())
        } else {
            ""
        }
    }
}
// #[cfg(test)]
// mod tests {
//     use crate::unity::world_manager::WorldManager;
//
//     #[ctor::ctor]
//     fn init_logger() {
//         use colored::Colorize;
//         use log::Level;
//         use std::io::Write;
//         env_logger::Builder::new()
//             .format_level(true)
//             .filter_level(log::LevelFilter::Debug)
//             .format(|buf, record| {
//                 writeln!(
//                     buf,
//                     "[{}:{}] {} [{}] {} ",
//                     // 文件名和行号（使用 `unwrap_or` 处理空值）
//                     record.file().unwrap_or("unknown"),
//                     record.line().unwrap_or(0),
//                     chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
//                     // 使用自定义颜色显示日志级别
//                     match record.level() {
//                         Level::Error => "ERROR".red().to_string(),
//                         Level::Warn => "WARN".yellow().to_string(),
//                         Level::Info => "INFO".green().to_string(),
//                         Level::Debug => "DEBUG".blue().to_string(),
//                         Level::Trace => "TRACE".purple().to_string(),
//                     },
//                     // 日志内容
//                     record.args(),
//                 )
//             })
//             .init();
//     }
//
//     #[test]
//     fn test1() {
//         let changed = WorldManager::change_scene("Assets/Scenes/RoomScene.unity");
//         println!("changed: {changed}");
//
//         for game_object in WorldManager::get().unwrap().game_objects.iter() {
//             let game_object = &*game_object.read().unwrap();
//             for children in game_object.children.iter() {
//                 let children = &*children.read().unwrap();
//
//                 // children.parent.clone().unwrap().get(|parent| {
//                 //     println!("{}", parent.name);
//                 // });
//
//                 if let Ok(children_transform) = children.transform.read() {
//                     children_transform.parent.clone().unwrap().get(|p| {
//                         println!("{}", p.position);
//                     });
//                 };
//                 // println!("{}", child.name);
//             }
//         }
//     }
// }
