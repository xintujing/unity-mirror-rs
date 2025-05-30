use crate::commons::revel_weak::RevelWeak;

trait SelfHandler<This, Args>: 'static {
    type Output;
    fn call(&self, s: &This, args: Args) -> Self::Output;
}

macro_rules! impl_self_handler ({ $($param:ident)* } => {
    impl<This, Func, Output, $($param,)*> SelfHandler<This, ($($param,)*)> for Func
    where
        This: 'static,
        Func: Fn(&This, $($param),*) -> Output + 'static,
    {
        type Output = Output;
        #[inline]
        #[allow(non_snake_case)]
        fn call(&self, this: &This, ($($param,)*): ($($param,)*)) -> Self::Output {
            (self)(this, $($param,)*)
        }
    }
});

impl_self_handler! {}
impl_self_handler! { A }
impl_self_handler! { A B }
impl_self_handler! { A B C }
impl_self_handler! { A B C D }
impl_self_handler! { A B C D E }
impl_self_handler! { A B C D E F }
impl_self_handler! { A B C D E F G }
impl_self_handler! { A B C D E F G H }
impl_self_handler! { A B C D E F G H I }
impl_self_handler! { A B C D E F G H I J }

pub struct SelfAction<Args, Return>(Box<dyn Fn(Args) -> Return>);

impl<Args, Return> Default for SelfAction<Args, Return> {
    fn default() -> Self {
        Self(Box::new(|_| panic!("Action called without a handler set")))
    }
}

impl<Args, Return> SelfAction<Args, Return> {
    pub fn new<F, This: 'static>(s: RevelWeak<Box<This>>, handler: F) -> Self
    where
        F: SelfHandler<This, Args, Output = Return>,
    {
        Self(Box::new(move |args| unsafe {
            handler.call(&**(s.upgrade().unwrap().get()), args)
        }))
    }
    pub fn call(&self, args: Args) -> Return {
        self.0(args)
    }
}
