use std::fmt::{Debug, Formatter};
use std::sync::{Mutex, MutexGuard};
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard, Weak};
use crate::commons;

pub struct Reference<T>(Weak<RwLock<T>>);

impl<T> Clone for Reference<T> {
    fn clone(&self) -> Self {
        Self { 0: self.0.clone() }
    }
}

impl<T> Debug for Reference<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Reference").field(&self.0).finish()
    }
}

impl<T> Default for Reference<T> {
    fn default() -> Self {
        Reference(Weak::new())
    }
}

impl<T> Reference<T> {
    pub fn new(w: Weak<RwLock<T>>) -> Self {
        Reference(w)
    }

    pub fn default() -> Self {
        Reference(Weak::new())
    }

    pub fn get(&self, f: impl Fn(RwLockReadGuard<T>)) {
        if let Some(value) = self.0.upgrade() {
            if let Ok(value) = value.read() {
                f(value)
            } else {
                log::warn!("Reference::get failed, value is locked")
            }
        } else {
            commons::trace::trace(4, "Reference::get failed, value is None".into());
        }
    }

    pub fn get_mut(&self, f: impl Fn(RwLockWriteGuard<T>)) {
        if let Some(value) = self.0.upgrade() {
            if let Ok(value) = value.write() {
                f(value)
            } else {
                log::warn!("Reference::get failed, value is locked")
            }
        } else {
            commons::trace::trace(4, "Reference::get_mut failed, value is None".into());
            // log::warn!("Reference::get failed, value is None")
        }
    }

    pub fn is_none(&self) -> bool {
        self.0.upgrade().is_none()
    }
}
