pub trait Namespace {
    fn get_namespace() -> &'static str
    where
        Self: Sized;
}
