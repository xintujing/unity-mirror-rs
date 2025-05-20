#[allow(unused)]
pub trait ComponentLifespan {
    fn awake(&self) {}
    fn update(&self) {}
    fn late_update(&self) {}
}
