use String;
use std::sync::MutexGuard;

pub struct NetworkIdentityState {}

#[derive(Debug, Default, Clone)]
pub struct NetworkIdentity {
    pub(crate) id: String,
}

impl NetworkIdentity {
    pub(crate) fn get_observers_len(&self) -> usize {
        0
    }
}

impl NetworkIdentity {
    pub(crate) fn state(id: &str) -> Option<MutexGuard<NetworkIdentityState>> {
        todo!()
    }
}
