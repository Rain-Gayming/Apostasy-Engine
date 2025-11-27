use downcast_rs::{Downcast, impl_downcast};

/// The id for a component
#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct ComponentId(pub u64);

/// A vector of different components types,
/// used for easier access to archetype information
#[derive(Clone, Hash, PartialEq, Eq)]
pub struct ComponentType(pub Vec<ComponentId>);

pub trait Component: Downcast {
    fn clone_box(&self) -> Box<dyn Component>;
}
impl_downcast!(Component);

impl Clone for Box<dyn Component> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}
