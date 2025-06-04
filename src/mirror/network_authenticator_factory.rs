use crate::commons::revel_arc::RevelArc;
use crate::mirror::Authenticator;
use once_cell::sync::Lazy;
use std::cell::RefCell;
use std::collections::HashMap;

static mut AUTHENTICATOR_FACTORY: Lazy<
    RefCell<HashMap<String, fn() -> RevelArc<Box<dyn Authenticator>>>>,
> = Lazy::new(|| RefCell::new(HashMap::new()));

pub struct AuthenticatorFactory;

impl AuthenticatorFactory {
    pub fn register<T: Authenticator + 'static>() {
        let full_name = T::get_full_name();
        #[allow(static_mut_refs)]
        unsafe {
            if AUTHENTICATOR_FACTORY.borrow().contains_key(full_name) {
                panic!("Authenticator {} is already registered", full_name);
            }
            let mut factory = || -> RevelArc<Box<dyn Authenticator>> {
                // 新建一个 authenticator 实例，并设置其弱引用
                let mut authenticator: RevelArc<Box<dyn Authenticator>> =
                    RevelArc::new(Box::new(T::new()));
                // clone authenticator to set weak self
                let clone_authenticator = authenticator.clone();
                // 设置弱引用
                authenticator.set_weak_self(clone_authenticator.downgrade());
                // 返回新建的 authenticator
                authenticator
            };
            AUTHENTICATOR_FACTORY
                .borrow_mut()
                .insert(full_name.to_string(), factory);
        }
    }
    pub fn create(full_name: &str) -> RevelArc<Box<dyn Authenticator>> {
        #[allow(static_mut_refs)]
        unsafe {
            match AUTHENTICATOR_FACTORY.borrow().get(full_name) {
                None => panic!("Authenticator {} is not registered", full_name),
                Some(factory) => factory(),
            }
        }
    }
}
