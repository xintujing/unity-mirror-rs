#![allow(dead_code)]

use crate::commons::object::Object;
use crate::mirror::network_connection::NetworkConnection;
use once_cell::sync::Lazy;
use std::any::Any;

static mut ON_SERVER_AUTHENTICATED: Lazy<Option<fn(&mut NetworkConnection)>> = Lazy::new(|| None);

pub trait AuthenticatorAnyMut {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: Authenticator + 'static> AuthenticatorAnyMut for T {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

pub trait Authenticator: Object //+ AuthenticatorAnyMut + MessageSerializer + MessageDeserializer
{
    fn new() -> Box<dyn Authenticator>
    where
        Self: Sized;
    fn on_start_server(&self) {}
    fn on_stop_server(&self) {}
    fn set_on_server_authenticated(&mut self, f: fn(connection: &mut NetworkConnection)) {
        #[allow(static_mut_refs)]
        unsafe {
            if ON_SERVER_AUTHENTICATED.is_some() {
                panic!("on_server_authenticated already set");
            }
            *ON_SERVER_AUTHENTICATED = Some(f);
        }
    }
    fn get_on_server_authenticated(&self) -> Option<&fn(&mut NetworkConnection)> {
        #[allow(static_mut_refs)]
        unsafe {
            ON_SERVER_AUTHENTICATED.as_ref()
        }
    }
    fn server_accept(&self, connection: &mut NetworkConnection) {
        if let Some(f) = self.get_on_server_authenticated() {
            f(connection);
        }
    }
    fn on_server_authenticate(&self, _conn: &mut NetworkConnection) {}
    fn server_reject(&self, conn: &mut NetworkConnection) {
        conn.disconnect()
    }
}
