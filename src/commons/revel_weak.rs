use crate::commons::revel_arc::RevelArc;
use std::cell::UnsafeCell;
use std::fmt::{Debug, Formatter};
use std::sync::{Arc, Weak};

pub struct RevelWeak<T: ?Sized>(pub Weak<UnsafeCell<T>>)
where
    T: Sized;

impl<T> RevelWeak<T> {
    pub fn from_raw(ptr: *const UnsafeCell<T>) -> RevelWeak<T> {
        let weak = unsafe { Weak::from_raw(ptr) };
        RevelWeak(weak)
    }
}

impl<T> Debug for RevelWeak<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<T> Clone for RevelWeak<T> {
    fn clone(&self) -> Self {
        RevelWeak(self.0.clone())
    }
}

impl<T> Default for RevelWeak<T> {
    fn default() -> Self {
        RevelWeak(std::sync::Weak::new())
    }
}

impl<T: 'static> RevelWeak<T> {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn get(&self) -> Option<&mut T> {
        if let Some(arc) = self.0.upgrade() {
            unsafe {
                return Some(&mut *arc.get());
            }
        }
        None
    }

    pub fn ptr_eq(&self, other: &RevelWeak<T>) -> bool {
        std::sync::Weak::ptr_eq(&self.0, &other.0)
    }

    pub fn as_ptr(&self) -> *const UnsafeCell<T> {
        self.0.as_ptr()
    }

    pub fn inner_ptr(&self) -> *const UnsafeCell<T> {
        self.0.as_ptr()
    }

    // pub fn eq_arc(&self, other: &RevelArc<T>) -> bool {
    //     if self.0.upgrade().is_none() {
    //         return false;
    //     }
    //     other.eq_weak(self)
    // }

    pub unsafe fn eq_value(&self, other: &T) -> bool {
        if self.0.upgrade().is_none() {
            return false;
        }

        RevelArc(self.0.upgrade().unwrap()).ptr_eq_value(other)
        // self.0.upgrade().unwrap().eq_value(other)
    }

    pub fn upgrade(&self) -> Option<Arc<UnsafeCell<T>>> {
        self.0.upgrade()
    }
    pub fn upgradable(&self) -> bool {
        self.0.upgrade().is_some()
    }
}
