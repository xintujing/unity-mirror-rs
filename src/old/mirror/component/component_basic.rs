use crate::mirror::component::component::Component;

#[allow(unused)]
pub trait ComponentBasic {
    fn id(&self) -> String;
    fn parent(&self) -> Option<Box<dyn Component>> {
        None
    }
    fn state_clear(&self);
}
