use crate::commons::action::SelfMutAction;
use crate::commons::RevelArc;
use crate::commons::RevelWeak;
use crate::metadata_settings::Metadata;
use crate::unity_engine::game_object::GameObject;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::AtomicIsize;
use std::sync::atomic::Ordering::SeqCst;

static mut WORLDS: Lazy<Vec<RevelArc<World>>> = Lazy::new(|| Vec::new());
static mut ACTIVE_WORLD_INDEX: AtomicIsize = AtomicIsize::new(-1);
static mut DONT_DESTROY_OBJECT: Lazy<HashMap<u64, RevelArc<GameObject>>> =
    Lazy::new(|| HashMap::default());

#[derive(Default)]
pub struct World {
    #[allow(unused)]
    scene_name: String,
    scene_path: String,
    game_objects: HashMap<u64, RevelArc<GameObject>>,
}

static mut SCENE_LOADED_ACTION: Lazy<SelfMutAction<(String, LoadSceneMode), ()>> =
    Lazy::new(|| SelfMutAction::default());

impl World {
    fn new(scene_path: &str) -> Self {
        match Metadata::get_scene(scene_path) {
            None => {
                panic!(
                    "Failed to load scene '{}'. Please check if the scene exists in the metadata.",
                    scene_path
                )
            }
            Some(scene_metadata) => {
                let mut world = World {
                    scene_name: scene_path.to_string(),
                    scene_path: scene_path.to_string(),
                    game_objects: Default::default(),
                };
                for (_, metadata_prefab) in scene_metadata.iter() {
                    let arc_game_object = GameObject::instance(metadata_prefab);
                    world
                        .game_objects
                        .insert(arc_game_object.id, arc_game_object);
                }
                world
            }
        }
    }

    fn destroy_all_game_object(&mut self) {
        for (_id, arc_game_object) in self.game_objects.iter_mut() {
            arc_game_object.on_disable();
        }
        for (_id, arc_game_object) in self.game_objects.iter_mut() {
            arc_game_object.on_destroy();
        }
    }

    fn destroy_game_object_with_id(&mut self, id: &u64) {
        if let Some(mut removed_arc_game_object) = self.game_objects.remove(id) {
            removed_arc_game_object.on_disable();
            removed_arc_game_object.on_destroy();
        }
    }

    pub fn add_game_object(&mut self, arc_game_object: RevelArc<GameObject>) {
        self.game_objects
            .insert(arc_game_object.id, arc_game_object);
    }

    pub fn get_scene_path(&self) -> String {
        return self.scene_path.clone();
    }
}

pub enum LoadSceneMode {
    Single,
    Additive,
}

pub struct WorldManagerStatic {
    loading: bool,
}

impl WorldManagerStatic {
    #[allow(unused)]
    pub(crate) fn loading(&self) -> bool {
        self.loading
    }
    pub(crate) fn set_loading(&mut self, loading: bool) {
        self.loading = loading;
    }
}

static mut WORLD_MANAGER_STATIC: Lazy<WorldManagerStatic> =
    Lazy::new(|| WorldManagerStatic { loading: false });

pub struct WorldManager;

impl Deref for WorldManager {
    type Target = WorldManagerStatic;

    fn deref(&self) -> &Self::Target {
        #[allow(static_mut_refs)]
        unsafe {
            &*WORLD_MANAGER_STATIC
        }
    }
}

impl DerefMut for WorldManager {
    fn deref_mut(&mut self) -> &mut Self::Target {
        #[allow(static_mut_refs)]
        unsafe {
            &mut *WORLD_MANAGER_STATIC
        }
    }
}

impl WorldManager {
    pub fn load_scene(scene_path: &str, mode: LoadSceneMode) -> usize {
        #[allow(static_mut_refs)]
        unsafe {
            Self.loading = true;
            let world = World::new(scene_path);
            let i = match mode {
                LoadSceneMode::Single => {
                    WORLDS
                        .iter_mut()
                        .for_each(|world| world.destroy_all_game_object());
                    WORLDS.clear();
                    WORLDS.push(RevelArc::new(world));
                    Self::set_active_scene(0)
                }
                LoadSceneMode::Additive => {
                    WORLDS.push(RevelArc::new(world));
                    WORLDS.len() - 1
                }
            };
            SCENE_LOADED_ACTION.call((scene_path.to_string(), mode));
            Self.loading = false;
            i
        }
    }

    pub fn set_scene_loaded(f: SelfMutAction<(String, LoadSceneMode), ()>) {
        #[allow(static_mut_refs)]
        unsafe {
            *SCENE_LOADED_ACTION = f;
        }
    }

    pub fn set_active_scene(index: usize) -> usize {
        #[allow(static_mut_refs)]
        unsafe {
            if index > WORLDS.len() {
                panic!("Invalid world index: {}", index);
            }
            ACTIVE_WORLD_INDEX.store(index as isize, SeqCst);
        }
        index
    }

    pub fn active_world() -> RevelWeak<World> {
        #[allow(static_mut_refs)]
        unsafe {
            let index = ACTIVE_WORLD_INDEX.load(SeqCst);
            if index >= 0 {
                if let Some(world) = WORLDS.get(index as usize) {
                    return world.downgrade();
                }
            }
            RevelWeak::default()
        }
    }

    pub fn dont_destroy_object(arc_game_object: RevelArc<GameObject>) -> RevelWeak<GameObject> {
        #[allow(static_mut_refs)]
        unsafe {
            let id = arc_game_object.id;

            let index = ACTIVE_WORLD_INDEX.load(std::sync::atomic::Ordering::SeqCst);

            if let Some(world) = WORLDS.get_mut(index as usize) {
                if world.game_objects.contains_key(&id) {
                    world.game_objects.remove(&id);
                }
            }

            if DONT_DESTROY_OBJECT.contains_key(&id) {
                panic!(
                    "GameObject with ID {} already exists in the dont_destroy_object world.",
                    id
                );
            }
            let weak_game_object = arc_game_object.downgrade();
            DONT_DESTROY_OBJECT.insert(id, arc_game_object);
            weak_game_object
        }
    }

    pub fn destroy(id: &u64) {
        #[allow(static_mut_refs)]
        unsafe {
            if let Some(mut arc_game_object) = DONT_DESTROY_OBJECT.remove(id) {
                arc_game_object.on_disable();
                arc_game_object.on_destroy();
            } else {
                let index = ACTIVE_WORLD_INDEX.load(std::sync::atomic::Ordering::SeqCst);
                if let Some(world) = WORLDS.get_mut(index as usize) {
                    world.destroy_game_object_with_id(id);
                }
            }
        }
    }

    pub fn root_game_objects() -> Vec<RevelWeak<GameObject>> {
        #[allow(static_mut_refs)]
        unsafe {
            let mut root_game_objects = vec![];

            let index = ACTIVE_WORLD_INDEX.load(SeqCst);
            if index >= 0 {
                if let Some(world) = WORLDS.get(index as usize) {
                    let world_root_game_objects = world
                        .game_objects
                        .values()
                        .map(|arc_game_object| arc_game_object.downgrade())
                        .collect::<Vec<_>>();
                    root_game_objects.extend(world_root_game_objects);
                }
            }

            let dont_destroy_objects = DONT_DESTROY_OBJECT
                .values()
                .filter(|arc_game_object| {
                    arc_game_object.parent.upgradable()
                    // if let Ok(game_object) = arc_game_object.read() {
                    //     game_object.parent.is_none()
                    // } else {
                    //     false
                    // }
                })
                .map(|arc_game_object| arc_game_object.downgrade())
                .collect::<Vec<_>>();
            root_game_objects.extend(dont_destroy_objects);
            root_game_objects
        }
    }
}

impl WorldManager {
    pub(super) fn fixed_update() {
        #[allow(static_mut_refs)]
        unsafe {
            for (_, game_object) in DONT_DESTROY_OBJECT.iter_mut() {
                game_object.fixed_update();
            }
            for world in WORLDS.iter_mut() {
                for (_, game_object) in world.game_objects.iter_mut() {
                    game_object.fixed_update();
                }
            }
        }
    }

    pub(super) fn update() {
        #[allow(static_mut_refs)]
        unsafe {
            for (_, game_object) in DONT_DESTROY_OBJECT.iter_mut() {
                game_object.update();
            }
            for world in WORLDS.iter_mut() {
                for (_, game_object) in world.game_objects.iter_mut() {
                    game_object.update();
                }
            }
        }
    }

    pub(super) fn late_update() {
        #[allow(static_mut_refs)]
        unsafe {
            for (_, game_object) in DONT_DESTROY_OBJECT.iter_mut() {
                game_object.late_update();
            }
            for world in WORLDS.iter_mut() {
                for (_, game_object) in world.game_objects.iter_mut() {
                    game_object.late_update();
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_change_scene() {
        // WorldManager::load_scene("Assets/Scenes/LobbyScene.unity", LoadSceneMode::Single);
        WorldManager::load_scene("Assets/Scenes/RoomScene.unity", LoadSceneMode::Single);
        // WorldManager::set_active_scene(1);
        // let network_room_player_prefab =
        //     Metadata::get_prefab("Assets/Prefabs/NetworkRoomPlayer.prefab").unwrap();
        //
        // let arc_game_object = GameObject::instance(&network_room_player_prefab);
        // let id = arc_game_object.read().unwrap().id;
        // WorldManager::dont_destroy_object(arc_game_object);

        // let vec = WorldManager::root_game_objects();
        // vec.iter().for_each(|weak_game_object| {
        //     if let Some(game_object) = weak_game_object.get() {
        //         println!("{}", game_object.name);
        //         if let Some(weak_network_identity) =
        //             game_object.try_get_component::<NetworkIdentity>()
        //         {
        //             println!("{}", weak_network_identity.net_id());
        //             // if let Some(read) = weak_network_identity.read() {
        //             //     println!("{}", read.net_id());
        //             // }
        //         }
        //     }
        // });

        // let mut looper = GameLooper::new();
        // looper.run();

        // WorldManager::change_scene("Assets/Scenes/LobbyScene.unity");
        // WorldManager::change_scene("Assets/Scenes/RoomScene.unity");
        // WorldManager::change_scene("Assets/Scenes/RoomScene.unity");
    }
}
