use crate::mirror::component::component::Component;

#[allow(unused)]
pub trait ComponentClone {
    fn clone_box(&self) -> Box<dyn Component>;
}

impl<T: Clone + Component + 'static> ComponentClone for T {
    fn clone_box(&self) -> Box<dyn Component> {
        Box::new(self.clone())
    }
}
