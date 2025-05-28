#![allow(dead_code)]

use crate::mirror::network_connection::NetworkConnection;
use crate::mirror::messages::message::{MessageDeserializer, MessageSerializer};
use once_cell::sync::Lazy;
use std::any::Any;

static mut AUTHENTICATOR_REGISTERS: Lazy<Option<fn() -> Box<dyn Authenticator>>> =
    Lazy::new(|| None);
static mut ON_SERVER_AUTHENTICATED: Lazy<Option<fn(&mut NetworkConnection)>> = Lazy::new(|| None);

pub fn register<T: Authenticator>() {
    #[allow(static_mut_refs)]
    unsafe {
        if AUTHENTICATOR_REGISTERS.is_some() {
            panic!("Authenticator already registered");
        }
        *AUTHENTICATOR_REGISTERS = Some(T::new);
    }
}

pub fn authenticator_factory() -> Option<&'static fn() -> Box<dyn Authenticator>> {
    #[allow(static_mut_refs)]
    unsafe {
        AUTHENTICATOR_REGISTERS.as_ref()
    }
}

#[macro_export]
macro_rules! authenticator_register {
    ($struct_name:path) => {
        paste::paste! {
            #[ctor::ctor]
            #[allow(non_snake_case)]
            fn [<$struct_name _registers>](){
                crate::mirror::authenticator::authenticator::register::<$struct_name>();
            }
        }
    };
}

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

pub trait Authenticator: AuthenticatorAnyMut + MessageSerializer + MessageDeserializer {
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
