use std::any::TypeId;

use anyhow::{Error, Result};

use crate::objects::component::Component;

#[derive(Clone, Copy, Debug)]
pub struct Voxel {}

pub struct VoxelDefinition {
    pub name: String,
    pub namespace: String,
    pub class: String,
    pub components: Vec<Box<dyn Component>>,
}

impl VoxelDefinition {
    /// Checks if the node has a component of type T
    pub fn has_component<T: Component + 'static>(&self) -> bool {
        self.components
            .iter()
            .any(|component| component.as_any().downcast_ref::<T>().is_some())
    }

    /// Gets a component of type T from the node
    pub fn get_component<T: Component + 'static>(&self) -> Result<&T> {
        self.components
            .iter()
            .find(|c| c.as_any().type_id() == TypeId::of::<T>())
            .and_then(|c| c.as_any().downcast_ref())
            .ok_or(Error::msg("No Comopnent of type"))
    }
}
