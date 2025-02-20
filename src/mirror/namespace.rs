pub trait Namespace {
    fn get_namespace() -> &'static str;
    fn get_prefix() -> &'static str;
}
