use crate::commons::object::Object;
use crate::commons::revel_weak::RevelWeak;
use crate::unity_engine::game_object::GameObject;
use std::any::Any;
use std::cell::UnsafeCell;
use std::ops::Deref;
use std::sync::Weak;

pub trait MonoBehaviour: Object + MonoBehaviourAny {
    fn awake(&mut self) {}
    fn on_enable(&mut self) {}
    fn on_validate(&mut self) {}
    fn start(&mut self) {}
    fn fixed_update(&mut self) {}
    fn update(&mut self) {}
    fn late_update(&mut self) {}
    fn on_disable(&mut self) {}
    fn on_destroy(&mut self) {}
}

pub trait MonoBehaviourAny {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn type_name(&self) -> String;
}

impl<T: MonoBehaviour + 'static> MonoBehaviourAny for T {
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
        format!("{}", std::any::type_name::<T>())
    }
}

impl<T: ?Sized + MonoBehaviour + 'static> RevelWeak<Box<T>> {
    pub fn to<TO: MonoBehaviour>(&self) -> RevelWeak<Box<TO>> {
        let raw = self.ptr();
        RevelWeak::from_raw(raw as *const UnsafeCell<Box<TO>>)
    }
}
