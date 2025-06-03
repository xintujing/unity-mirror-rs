pub trait MonoBehaviour {
    fn awake(&self) {}
    fn on_enable(&self) {}
    fn reset(&self) {}
    fn start(&self) {}
    fn fixed_update(&self) {}
    fn update(&self) {}
    fn late_update(&self) {}
    fn on_disable(&self) {}
    fn on_destroy(&self) {}
}
