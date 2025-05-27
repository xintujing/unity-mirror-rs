use std::any::Any;

pub type VirtualFunc = Box<dyn Fn(Vec<Box<dyn Any>>) -> Option<Box<dyn Any>>>;

fn wrap_func(func: fn(i32) -> i32) -> VirtualFunc {
    Box::new(move |args| {
        let arg = args[0].downcast_ref::<i32>().unwrap();
        let result = func(*arg);
        Some(Box::new(result))
    })
}
