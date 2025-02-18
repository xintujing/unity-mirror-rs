use std::fmt::Debug;

// NetworkBehaviourTrait
pub trait NetworkBehaviourTrait: Debug + Send + Sync {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;

    // fn downcast_mut<T>(&mut self) -> T {
    //     self.as_any_mut().downcast_mut::<T>().unwrap()
    // }
}

// Value for NetworkBehaviours type
pub type NetworkBehaviourType = Box<dyn NetworkBehaviourTrait>;

#[derive(Debug)]
pub struct NetworkBehaviour {}

impl NetworkBehaviour {}

#[cfg(test)]
mod tests {}
