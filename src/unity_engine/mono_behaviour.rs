use crate::commons::object::Object;
use crate::commons::revel_weak::RevelWeak;
use std::any::Any;

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
    // fn downcast<TO>(&self) -> Option<&TO>;
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
    pub fn downcast<TO: Any>(&self) -> Option<&RevelWeak<Box<TO>>> {
        if self.get()?.as_any().downcast_ref::<TO>().is_none() {
            return None;
        }
        Some(unsafe { &*(self as *const dyn Any as *const RevelWeak<Box<TO>>) })
    }

    pub fn parallel<TO: Any>(&self) -> Option<&RevelWeak<TO>> {
        Some(unsafe { &*(self as *const dyn Any as *const RevelWeak<TO>) })
    }
}
