use crate::commons::revel_arc::RevelArc;
use crate::commons::revel_weak::RevelWeak;
use crate::metadata_settings::unity::metadata_component::MetadataComponentWrapper;
use crate::metadata_settings::unity::metadata_prefab::MetadataPrefab;
use crate::unity_engine::mono_behaviour::MonoBehaviour;
use crate::unity_engine::mono_behaviour_factory::MonoBehaviourFactory;
use crate::unity_engine::transform::Transform;
use crate::unity_engine::WorldManager;
use once_cell::sync::Lazy;
use rand::RngCore;
use std::any::TypeId;
use std::collections::HashMap;

static mut COMPONENT_LOADING: Lazy<Vec<(RevelWeak<GameObject>, MetadataComponentWrapper)>> =
    Lazy::new(|| vec![]);

fn append_component_loading(
    weak_game_object: RevelWeak<GameObject>,
    component: MetadataComponentWrapper,
) {
    #[allow(static_mut_refs)]
    unsafe {
        COMPONENT_LOADING.push((weak_game_object, component));
    }
}

fn component_loading() {
    #[allow(static_mut_refs)]
    unsafe {
        COMPONENT_LOADING.reverse();
        while let Some((arc_game_object, metadata_prefab)) = COMPONENT_LOADING.pop() {
            GameObject::load_component(arc_game_object, &metadata_prefab);
        }
    }
}

#[derive(Default)]
pub struct GameObject {
    pub id: u64,
    pub name: String,
    pub asset_id: u32,
    pub asset_path: String,
    pub tag: String,
    pub layer: i32,
    pub is_static: bool,
    pub is_active: bool,
    pub transform: RevelArc<Transform>,
    pub parent: RevelWeak<GameObject>,
    pub children: HashMap<u64, RevelArc<GameObject>>,
    // pub components: IndexMap<TypeId, RevelArc<Box<dyn MonoBehaviour>>>>,
    component_mapping: HashMap<TypeId, Vec<usize>>,
    // pub components: Vec<RevelArc<Box<dyn MonoBehaviour>>>>,
    pub components: Vec<Vec<RevelArc<Box<dyn MonoBehaviour>>>>,
}

impl GameObject {
    pub fn instance(metadata_prefab: &MetadataPrefab) -> RevelArc<GameObject> {
        let arc_game_object = Self::new(RevelWeak::default(), metadata_prefab);
        Self::recursive_children(arc_game_object.downgrade(), &metadata_prefab.children);
        component_loading();
        arc_game_object
    }

    pub fn instantiate(metadata_prefab: &MetadataPrefab) {
        let arc_game_object = Self::instance(metadata_prefab);
        if let Some(world) = WorldManager::active_world().get() {
            world.add_game_object(arc_game_object);
        }
    }

    pub fn default() -> GameObject {
        Self {
            id: rand::rng().next_u64(),
            ..Default::default()
        }
    }

    pub fn new(
        parent: RevelWeak<GameObject>,
        metadata_prefab: &MetadataPrefab,
    ) -> RevelArc<GameObject> {
        // 随机数
        let mut rng = rand::rng();
        println!("new game object: {}", metadata_prefab.name);
        let mut game_object = GameObject {
            id: rng.next_u64(),
            name: metadata_prefab.name.clone(),
            tag: metadata_prefab.tag.clone(),
            layer: metadata_prefab.layer,
            is_static: metadata_prefab.is_static,
            is_active: metadata_prefab.is_active,
            asset_id: metadata_prefab.asset_id,
            asset_path: metadata_prefab.asset_path.clone(),
            parent: parent.clone(),
            components: Default::default(),
            transform: Default::default(),
            children: HashMap::default(),
            component_mapping: Default::default(),
        };

        let mut transform = Transform::new_with_metadata(&metadata_prefab.transform);

        if let Some(parent_game_object) = parent.get() {
            transform.parent = parent_game_object.transform.downgrade()
        }

        game_object.transform = RevelArc::new(transform);
        let mut arc_game_object = RevelArc::new(game_object);

        append_component_loading(
            arc_game_object.downgrade(),
            metadata_prefab.components.clone(),
        );

        arc_game_object.transform.game_object = arc_game_object.downgrade();
        arc_game_object
    }

    fn recursive_children(parent: RevelWeak<GameObject>, children: &Vec<MetadataPrefab>) {
        for metadata_prefab in children {
            let arc_game_object = Self::new(parent.clone(), metadata_prefab);

            let weak_game_object = arc_game_object.downgrade();

            if let Some(parent) = parent.get() {
                parent.children.insert(arc_game_object.id, arc_game_object);
            }

            if metadata_prefab.children.len() > 0 {
                Self::recursive_children(weak_game_object.clone(), &metadata_prefab.children);
            }

            if let Some(game_object) = weak_game_object.get() {
                for (_, children_game_object) in &game_object.children {
                    game_object
                        .transform
                        .children
                        .push(children_game_object.transform.downgrade());
                }
            }
        }
    }

    fn load_component(
        weak_game_object: RevelWeak<GameObject>,
        component: &MetadataComponentWrapper,
    ) {
        for (full_path, component_settings) in component.group_by_full_name() {
            let mono_behaviour_chain = MonoBehaviourFactory::create(
                &full_path,
                weak_game_object.clone(),
                &component_settings,
            );

            let mut mono_behaviours = vec![];
            for (mono_behaviour, type_id) in mono_behaviour_chain {
                let arc_mono_behaviour = RevelArc::new(mono_behaviour);
                mono_behaviours.push((arc_mono_behaviour, type_id));
            }

            if let Some(game_object) = weak_game_object.get() {
                game_object.add_component(mono_behaviours);
            }
        }
    }
}

impl GameObject {
    pub fn add_component(
        &mut self,
        mono_behaviour_chain: Vec<(RevelArc<Box<dyn MonoBehaviour + 'static>>, TypeId)>,
    ) {
        if mono_behaviour_chain.len() == 0 {
            panic!("MonoBehaviourChain is empty");
        }
        let index = self.components.len();
        let mut arc_mono_behaviours = Vec::new();
        for (mono_behaviour, type_id) in mono_behaviour_chain {
            arc_mono_behaviours.push(mono_behaviour);
            if !self.component_mapping.contains_key(&type_id) {
                self.component_mapping.insert(type_id, vec![index]);
            } else {
                if let Some(mapping) = self.component_mapping.get_mut(&type_id) {
                    mapping.push(index);
                };
            }
        }
        if let Some(mono_behaviour) = arc_mono_behaviours.last_mut() {
            mono_behaviour.awake();
        }
        self.components.push(arc_mono_behaviours);
    }

    pub fn try_get_component<T: MonoBehaviour + 'static>(
        &self,
    ) -> Option<RevelWeak<Box<dyn MonoBehaviour>>> {
        let type_id = TypeId::of::<T>();

        let vec_index = self
            .component_mapping
            .get(&type_id)
            .cloned()
            .unwrap_or(vec![]);
        if vec_index.len() == 0 {
            return None;
        }

        Some(
            self.components
                .get(vec_index[0])?
                .last()
                .cloned()?
                .downgrade(),
        )
    }

    // pub fn find_component<T: MonoBehaviour>(
    //     &self,
    //     t: impl Any,
    // ) -> Option<RevelWeak<Box<dyn MonoBehaviour>>> {
    //     let x = t as *const dyn MonoBehaviour;
    //     // let x1 = x.eq(&self.components.as_ptr());
    //
    //     for component in self.components.iter() {
    //         let x2 = component.last().unwrap().as_ref();
    //         let x3 = x2 as *const dyn MonoBehaviour;
    //         if x == x3 {
    //             return Some(component.last().unwrap().downgrade());
    //         }
    //     }
    //     None
    // }

    pub fn find_transform(&self, instance_id: &i32) -> Option<RevelWeak<Transform>> {
        if self.transform.instance_id == *instance_id {
            return Some(self.transform.downgrade());
        }
        for (_, children_game_object) in self.children.iter() {
            if let Some(weak_transform) = children_game_object.find_transform(instance_id) {
                return Some(weak_transform);
            }
        }
        None
    }
}

//调用了start的组件map
static mut STARTED_COMPONENT: Vec<RevelWeak<Box<dyn MonoBehaviour>>> = Vec::new();

macro_rules! recursive_event_fn {
    ($($fn_name:ident),*) => {
        $(
            pub(crate) fn $fn_name(&mut self) {
                for (_, children_game_object) in self.children.iter_mut() {
                     children_game_object.$fn_name()
                }
                for component in &mut self.components {
                    if let Some(component_mut) = component.last_mut() {
                        if let Some(component_mut) = component.last_mut() {
                            component_mut.get().$fn_name();
                        }
                    }
                }
            }
        )*
    };
}

#[allow(unused)]
impl GameObject {
    recursive_event_fn!(
        awake,
        on_enable,
        on_validate,
        start,
        fixed_update,
        late_update,
        on_disable
    );

    pub(crate) fn update(&mut self) {
        for (_, children_game_object) in self.children.iter_mut() {
            children_game_object.update()
        }
        for component in &mut self.components {
            #[allow(static_mut_refs)]
            unsafe {
                let started = STARTED_COMPONENT.iter().any(|weak_component| {
                    weak_component.ptr_eq(&component.last_mut().unwrap().downgrade())
                });
                if !started {
                    component.last_mut().unwrap().start();
                    STARTED_COMPONENT.push(component.last_mut().unwrap().downgrade())
                }
                component.last_mut().unwrap().update()
            }
        }
    }

    pub(crate) fn on_destroy(&mut self) {
        for (_, children_game_object) in self.children.iter_mut() {
            children_game_object.on_destroy()
        }
        for component in &mut self.components {
            #[allow(static_mut_refs)]
            unsafe {
                component.last_mut().unwrap().on_destroy();
                STARTED_COMPONENT.retain(|weak_component| {
                    if weak_component.ptr_eq(&component.last_mut().unwrap().downgrade()) {
                        return false;
                    }
                    true
                });
            }
        }
    }
}
