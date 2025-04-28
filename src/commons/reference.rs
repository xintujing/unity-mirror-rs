use std::fmt::{Debug, Formatter};
use std::sync::Weak;
use std::sync::{Mutex, MutexGuard};

pub struct Reference<T>(Weak<Mutex<T>>);

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
    pub fn new(w: Weak<Mutex<T>>) -> Self {
        Reference(w)
    }

    pub fn default() -> Self {
        Reference(Weak::new())
    }

    pub fn get(&self, f: impl Fn(MutexGuard<T>)) {
        if let Some(value) = self.0.upgrade() {
            if let Ok(value) = value.try_lock() {
                f(value)
            } else {
                log::warn!("Reference::get failed, value is locked")
            }
        } else {
            log::warn!("Reference::get failed, value is None")
        }
    }
}
