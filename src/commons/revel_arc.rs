use crate::commons::revel_weak::RevelWeak;
use std::cell::UnsafeCell;
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

pub struct RevelArc<T>(pub(super) Arc<UnsafeCell<T>>);

impl<T> Debug for RevelArc<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<T> RevelArc<T> {
    pub fn into_inner(self) -> T {
        Arc::into_inner(self.0).unwrap().into_inner()
    }
}

impl<T: Eq> Eq for RevelArc<T> {}
impl<T: PartialEq> PartialEq<Self> for RevelArc<T> {
    fn eq(&self, other: &Self) -> bool {
        unsafe { (&*self.0.get()).eq(&*other.0.get()) }
    }
}

impl<T: Hash + 'static> Hash for RevelArc<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.get().hash(state);
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
        Self(Arc::new(UnsafeCell::new(Default::default())))
    }
}

impl<T> RevelArc<T> {
    pub fn new(value: T) -> Self {
        RevelArc(Arc::new(UnsafeCell::new(value)))
    }

    pub fn downgrade(&self) -> RevelWeak<T> {
        RevelWeak(Arc::downgrade(&self.0))
    }

    pub fn ptr_eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }

    pub fn ptr_eq_weak(&self, other: &RevelWeak<T>) -> bool {
        if other.0.upgrade().is_none() {
            return false;
        }
        self.ptr_eq(&Self(other.0.upgrade().unwrap()))
    }

    pub unsafe fn ptr_eq_value(&self, other: &T) -> bool {
        let a_ptr = other as *const T;
        let b_uc_ptr = Arc::into_raw(self.0.clone());
        let offset = size_of::<UnsafeCell<T>>() - size_of::<T>();
        (b_uc_ptr as *const u8).add(offset) as *const T == a_ptr
    }

    pub fn as_ptr(&self) -> *const UnsafeCell<T> {
        Arc::as_ptr(&self.0)
    }
}

// pub trait VecRevelArc {
//     fn last_to_weak<T: MonoBehaviour>(&self) -> Option<RevelWeak<Box<T>>>;
// }
