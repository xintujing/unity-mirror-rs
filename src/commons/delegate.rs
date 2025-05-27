// use std::any::Any;
//
// pub struct Delegate<Args, F, R>
// where
//     Args: DelegateArgs,
//     F: Fn(Vec<Box<dyn Any>>) -> Option<Box<dyn Any>>,
//     R: Any + 'static,
// {
//     f: F,
//     phantom: std::marker::PhantomData<Args>,
// }
//
// pub trait DelegateArgs: Sized {
//     fn from() {}
// }
