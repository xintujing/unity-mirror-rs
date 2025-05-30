use crate::mirror::Authenticator;
use once_cell::sync::Lazy;
use std::cell::RefCell;
use std::collections::HashMap;

static mut AUTHENTICATOR_FACTORY: Lazy<RefCell<HashMap<String, fn() -> Box<dyn Authenticator>>>> =
    Lazy::new(|| RefCell::new(HashMap::new()));

pub struct AuthenticatorFactory;

impl AuthenticatorFactory {
    pub fn register<T: Authenticator>() {
        let full_name = T::get_full_name();
        #[allow(static_mut_refs)]
        unsafe {
            if AUTHENTICATOR_FACTORY.borrow().contains_key(full_name) {
                panic!("Authenticator {} is already registered", full_name);
            }
            AUTHENTICATOR_FACTORY
                .borrow_mut()
                .insert(full_name.to_string(), T::new);
        }
    }
    pub fn create(full_name: &str) -> Box<dyn Authenticator> {
        #[allow(static_mut_refs)]
        unsafe {
            match AUTHENTICATOR_FACTORY.borrow().get(full_name) {
                None => panic!("Authenticator {} is not registered", full_name),
                Some(factory) => factory(),
            }
        }
    }
}
