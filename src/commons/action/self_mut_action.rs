use crate::commons::revel_weak::RevelWeak;
use std::mem;

pub trait SelfMutHandler<This, Args>: 'static {
    type Output;
    fn call(&self, s: &mut This, args: Args) -> Self::Output;
}

macro_rules! impl_self_mut_handler ({ $($param:ident)* } => {
    impl<This, Func, Output, $($param,)*> SelfMutHandler<This, ($($param,)*)> for Func
    where
        This: 'static,
        Func: Fn(&mut This, $($param),*) -> Output + 'static,
    {
        type Output = Output;
        #[inline]
        #[allow(non_snake_case)]
        fn call(&self, this: &mut This, ($($param,)*): ($($param,)*)) -> Self::Output {
            (self)(this, $($param,)*)
        }
    }
});

impl_self_mut_handler! {}
impl_self_mut_handler! { A }
impl_self_mut_handler! { A B }
impl_self_mut_handler! { A B C }
impl_self_mut_handler! { A B C D }
impl_self_mut_handler! { A B C D E }
impl_self_mut_handler! { A B C D E F }
impl_self_mut_handler! { A B C D E F G }
impl_self_mut_handler! { A B C D E F G H }
impl_self_mut_handler! { A B C D E F G H I }
impl_self_mut_handler! { A B C D E F G H I J }

pub struct SelfMutAction<Args, Return> {
    f: Box<dyn Fn(Args) -> Return>,
    reg: bool,
}

impl<Args, Return> Default for SelfMutAction<Args, Return> {
    fn default() -> Self {
        unsafe {
            Self {
                f: Box::new(|_| mem::zeroed()),
                reg: false,
            }
        }
    }
}

impl<Args, Return> SelfMutAction<Args, Return> {
    pub fn new<F, This: 'static>(s: RevelWeak<Box<This>>, handler: F) -> Self
    where
        F: SelfMutHandler<This, Args, Output=Return>,
    {
        Self {
            f: Box::new(move |args| handler.call(&mut **(s.upgrade().unwrap()), args)),
            reg: true,
        }
    }
    pub fn call(&self, args: Args) -> Return {
        (self.f)(args)
    }

    pub fn is_registered(&self) -> bool {
        self.reg
    }

    pub fn reset(&mut self) {
        *self = Self::default()
    }
}
