use std::any::TypeId;

use anyhow::{Error, Result};
use apostasy_macros::Resource;
use hashbrown::HashMap;

use crate::objects::component::{BoxedComponent, Component};

#[derive(Clone, Copy, Debug)]
pub struct Voxel {
    pub id: VoxelId,
}

#[derive(Clone)]
pub struct VoxelDefinition {
    pub name: String,
    pub namespace: String,
    pub class: String,
    pub components: Vec<BoxedComponent>,
}

impl std::fmt::Debug for VoxelDefinition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VoxelDefinition")
            .field("name", &self.name)
            .field("namespace", &self.namespace)
            .field("class", &self.class)
            .field("component_count", &self.components.len())
            .finish()
    }
}

impl VoxelDefinition {
    /// Checks if the voxel has a component of type T
    pub fn has_component<T: Component + 'static>(&self) -> bool {
        self.components
            .iter()
            .any(|component| component.as_any().downcast_ref::<T>().is_some())
    }

    /// Gets a component of voxel T from the node
    pub fn get_component<T: Component + 'static>(&self) -> Result<&T> {
        self.components
            .iter()
            .find(|c| c.as_any().type_id() == TypeId::of::<T>())
            .and_then(|c| c.as_any().downcast_ref())
            .ok_or(Error::msg("No Comopnent of type"))
    }
}

pub type VoxelId = u16;

#[derive(Resource, Clone, Debug)]
pub struct VoxelRegistry {
    pub defs: Vec<VoxelDefinition>,
    pub name_to_id: HashMap<String, VoxelId>,
    pub id_to_name: HashMap<VoxelId, String>,
}

impl Default for VoxelRegistry {
    fn default() -> Self {
        VoxelRegistry::new()
    }
}

impl VoxelRegistry {
    pub fn new() -> Self {
        let mut defs = Vec::new();
        let mut name_to_id = HashMap::new();
        let mut id_to_name = HashMap::new();

        // reserve id 0 for air
        defs.push(VoxelDefinition {
            name: "Air".to_string(),
            namespace: "Apostasy".to_string(),
            class: "Voxel".to_string(),
            components: vec![],
        });
        name_to_id.insert("Apostasy:Air".to_string(), 0);
        id_to_name.insert(0, "Apostasy:Air".to_string());

        Self {
            defs,
            name_to_id,
            id_to_name,
        }
    }
}
