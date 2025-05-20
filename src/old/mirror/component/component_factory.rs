use crate::metadata_settings::mirror::network_behaviours::metadata_network_behaviour::MetadataNetworkBehaviourWrapper;
use crate::mirror::component::component::Component;

#[allow(unused)]
pub trait ComponentFactory {
    fn instance(settings: &MetadataNetworkBehaviourWrapper) -> Box<dyn Component>
    where
        Self: Sized;
}

impl<T: Component + 'static> ComponentFactory for T {
    fn instance(settings: &MetadataNetworkBehaviourWrapper) -> Box<dyn Component>
    where
        Self: Sized,
    {
        Box::new(Self::new(settings, &mut 0, &mut 0))
    }
}
