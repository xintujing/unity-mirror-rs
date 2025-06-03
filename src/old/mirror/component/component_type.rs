use crate::mirror::component::component::Component;
use std::any::Any;

#[allow(unused)]
pub trait ComponentType {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn type_name(&self) -> String;
}

impl<T: Component + 'static> ComponentType for T {
    fn as_any(&self) -> &dyn Any
    where
        Self: Sized,
    {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any
    where
        Self: Sized,
    {
        self
    }

    fn type_name(&self) -> String {
        std::any::type_name::<T>().to_string()
    }
}
