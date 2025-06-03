#![allow(dead_code)]

use crate::commons::object::Object;
use crate::mirror::network_connection::NetworkConnection;
use once_cell::sync::Lazy;
use std::any::Any;
use crate::commons::action::SelfMutAction;
use crate::commons::revel_arc::RevelArc;
use crate::commons::revel_weak::RevelWeak;
use crate::unity_engine::MonoBehaviour;

static mut ON_SERVER_AUTHENTICATED: Lazy<Option<fn(&mut NetworkConnection)>> = Lazy::new(|| None);

// pub trait AuthenticatorAnyMut {
//     fn as_any(&self) -> &dyn Any;
//     fn as_any_mut(&mut self) -> &mut dyn Any;
// }
//
// impl<T: Authenticator + 'static> AuthenticatorAnyMut for T {
//     fn as_any(&self) -> &dyn Any {
//         self
//     }
//
//     fn as_any_mut(&mut self) -> &mut dyn Any {
//         self
//     }
// }

pub trait Authenticator: MonoBehaviour {
    fn new() -> Box<dyn Authenticator>
    where
        Self: Sized;
    fn on_start_server(&self) {}
    fn on_stop_server(&self) {}
    fn set_on_server_authenticated(&mut self, event: SelfMutAction<(RevelArc<NetworkConnection>,), ()>);
    fn get_on_server_authenticated(&self, f: Box<dyn Fn(&SelfMutAction<(RevelArc<NetworkConnection>,), ()>)>);
    fn server_accept(&self, connection: RevelArc<NetworkConnection>) {
        // self.get_on_server_authenticated(Box::new(|f| {
        //     let connection_clone = connection.clone();
        //     f.call((connection_clone,));
        // }));
        // // self.get_on_server_authenticated(|f| {
        // //     f.call((connection,))
        // // });
    }
    fn on_server_authenticate(&self, connection: RevelArc<NetworkConnection>) {}
    fn server_reject(&self, conn: &mut NetworkConnection) {
        conn.disconnect()
    }

    fn set_weak_self(&mut self, weak_self: RevelWeak<Box<dyn Authenticator>>);
}
