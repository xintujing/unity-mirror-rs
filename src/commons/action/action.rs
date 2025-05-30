trait Handler<Args>: 'static {
    type Output;
    fn call(&self, args: Args) -> Self::Output;
}

macro_rules! impl_handler ({ $($param:ident)* } => {
    impl<Func, Output, $($param,)*> Handler<($($param,)*)> for Func
    where
        Func: Fn($($param),*) -> Output + 'static,
    {
        type Output = Output;
        #[inline]
        #[allow(non_snake_case)]
        fn call(&self, ($($param,)*): ($($param,)*)) -> Self::Output {
            (self)($($param,)*)
        }
    }
});

impl_handler! {}
impl_handler! { A }
impl_handler! { A B }
impl_handler! { A B C }
impl_handler! { A B C D }
impl_handler! { A B C D E }
impl_handler! { A B C D E F }
impl_handler! { A B C D E F G }
impl_handler! { A B C D E F G H }
impl_handler! { A B C D E F G H I }
impl_handler! { A B C D E F G H I J }

pub struct Action<Args, Return>(Box<dyn Fn(Args) -> Return>);

impl<Args, Return> Default for Action<Args, Return> {
    fn default() -> Self {
        Self(Box::new(|_| panic!("Action called without a handler set")))
    }
}

impl<Args, Return> Action<Args, Return> {
    pub fn new<F>(handler: F) -> Self
    where
        F: Handler<Args, Output = Return>,
    {
        Self(Box::new(move |args| unsafe { handler.call(args) }))
    }
    pub fn call(&self, args: Args) -> Return {
        self.0(args)
    }
}
