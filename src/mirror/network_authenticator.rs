#![allow(dead_code)]

use crate::commons::action::SelfMutAction;
use crate::commons::revel_arc::RevelArc;
use crate::commons::revel_weak::RevelWeak;
use crate::mirror::network_connection::NetworkConnection;
use crate::unity_engine::MonoBehaviour;

pub trait Authenticator: MonoBehaviour {
    fn new() -> Box<dyn Authenticator>
    where
        Self: Sized;
    fn on_start_server(&self) {}
    fn on_stop_server(&self) {}
    fn set_on_server_authenticated(
        &mut self,
        event: SelfMutAction<(RevelArc<NetworkConnection>,), ()>,
    );
    fn get_on_server_authenticated(&self) -> &SelfMutAction<(RevelArc<NetworkConnection>,), ()>;
    fn server_accept(&self, connection: RevelArc<NetworkConnection>) {
        self.get_on_server_authenticated().call((connection,));
    }
    fn on_server_authenticate(&self, connection: RevelArc<NetworkConnection>) {}
    fn server_reject(&self, conn: &mut NetworkConnection) {
        conn.disconnect()
    }

    fn set_weak_self(&mut self, weak_self: RevelWeak<Box<dyn Authenticator>>);
}
