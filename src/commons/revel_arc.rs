use crate::commons::revel_weak::RevelWeak;
use crate::unity_engine::MonoBehaviour;
use std::fmt::{Debug, Formatter};
use std::ops::{Deref, DerefMut};

pub struct RevelArc<T>(pub(super) std::sync::Arc<std::cell::UnsafeCell<T>>);

impl<T> Debug for RevelArc<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<T> Clone for RevelArc<T> {
    fn clone(&self) -> Self {
        RevelArc(self.0.clone())
    }
}

impl<T> Deref for RevelArc<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &mut *self.0.get() }
    }
}

impl<T> DerefMut for RevelArc<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.0.get() }
    }
}

impl<T: Default> Default for RevelArc<T> {
    fn default() -> Self {
        Self(std::sync::Arc::new(std::cell::UnsafeCell::new(
            Default::default(),
        )))
    }
}

impl<T> RevelArc<T> {
    pub fn new(value: T) -> Self {
        RevelArc(std::sync::Arc::new(std::cell::UnsafeCell::new(value)))
    }

    pub fn downgrade(&self) -> RevelWeak<T> {
        RevelWeak(std::sync::Arc::downgrade(&self.0))
    }

    pub fn ptr_eq(&self, other: &Self) -> bool {
        std::sync::Arc::ptr_eq(&self.0, &other.0)
    }

    pub fn ptr_eq_weak(&self, other: &RevelWeak<T>) -> bool {
        if other.0.upgrade().is_none() {
            return false;
        }
        self.ptr_eq(&Self(other.0.upgrade().unwrap()))
    }

    // pub fn to<TT>(&self) -> RevelArc<Box<TT>> {
    //     let ptr = Arc::as_ptr(&self.0);
    //     let n_ptr = ptr as *const UnsafeCell<Box<TT>>;
    //     unsafe { RevelArc(Arc::from_raw(n_ptr)) }
    // }

    pub unsafe fn ptr_eq_value(&self, other: &T) -> bool {
        let a_ptr = other as *const T;
        let b_uc_ptr = std::sync::Arc::into_raw(self.0.clone());
        let offset = size_of::<std::cell::UnsafeCell<T>>() - size_of::<T>();
        (b_uc_ptr as *const u8).add(offset) as *const T == a_ptr
    }
}

pub trait VecRevelArc {
    fn last_to_weak<T: MonoBehaviour>(&self) -> Option<RevelWeak<Box<T>>>;
}
// impl VecRevelArc for Vec<(RevelArc<Box<dyn MonoBehaviour>>, TypeId)> {
//     fn last_to_weak<T: MonoBehaviour>(&self) -> Option<RevelWeak<Box<T>>> {
//         if let Some((mono_behaviour, _)) = self.last() {
//             return Some(mono_behaviour.clone().downgrade().to::<T>());
//         }
//         None
//     }
// }
