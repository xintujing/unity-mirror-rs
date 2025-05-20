pub trait Object {
    fn get_full_name() -> &'static str
    where
        Self: Sized;
}
