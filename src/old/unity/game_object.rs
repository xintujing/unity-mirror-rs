use crate::commons::reference::Reference;
use crate::unity::mono_behaviour::MonoBehaviour;
use crate::unity::Transform;
use std::any::TypeId;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub struct GameObject {
    pub(super) id: String,
    pub(super) name: String,
    pub(super) tag: String,
    pub(super) layer: i32,
    pub(super) is_static: bool,
    pub(super) is_active: bool,
    pub(super) asset_id: u32,
    pub(super) asset_path: String,
    pub(super) parent: Option<Reference<GameObject>>,
    pub(super) components: HashMap<TypeId, Box<dyn MonoBehaviour>>,
    pub(super) transform: Arc<RwLock<Transform>>,
    pub(super) children: Vec<Arc<RwLock<GameObject>>>,
}
