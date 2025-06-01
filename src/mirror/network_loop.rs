use std::ops::{Deref, DerefMut};
use once_cell::sync::Lazy;
use crate::commons::action::SelfMutAction;
use crate::mirror::NetworkServer;

static mut NETWORK_LOOP_STATIC: Lazy<NetworkLoopStatic> = Lazy::new(|| NetworkLoopStatic {
    on_early_update: vec![],
    on_late_update: vec![],
});
pub struct NetworkLoopStatic {
    on_early_update: Vec<SelfMutAction<(), ()>>,
    on_late_update: Vec<SelfMutAction<(), ()>>,
}

impl NetworkLoopStatic {
    pub fn append_early_update_handler(&mut self, handler: SelfMutAction<(), ()>) {
        self.on_early_update.push(handler);
    }

    pub fn append_late_update_handler(&mut self, handler: SelfMutAction<(), ()>) {
        self.on_late_update.push(handler);
    }
}

pub struct NetworkLoop;

impl Deref for NetworkLoop {
    type Target = NetworkLoopStatic;

    fn deref(&self) -> &Self::Target {
        #[allow(static_mut_refs)]
        unsafe { &NETWORK_LOOP_STATIC }
    }
}

impl DerefMut for NetworkLoop {
    fn deref_mut(&mut self) -> &mut Self::Target {
        #[allow(static_mut_refs)]
        unsafe { &mut NETWORK_LOOP_STATIC }
    }
}
impl NetworkLoop {
    pub fn network_early_update(&self) {
        NetworkServer::network_early_update();
        for on_early_update in self.on_early_update.iter() {
            on_early_update.call(())
        }
    }
    pub fn network_late_update(&self) {
        for on_late_update in self.on_late_update.iter() {
            on_late_update.call(())
        }
        NetworkServer::network_late_update();
    }
}