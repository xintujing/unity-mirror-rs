use std::fmt::Debug;

// NetworkBehaviourTrait
pub trait NetworkBehaviourTrait: Debug + Send + Sync {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

// Value for NetworkBehaviours type
pub type NetworkBehaviourType = Box<dyn NetworkBehaviourTrait>;

#[derive(Debug)]
pub struct NetworkBehaviour {}

impl NetworkBehaviour {}

#[cfg(test)]
mod tests {}
