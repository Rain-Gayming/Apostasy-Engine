use std::sync::Arc;

use crate::engine::voxels::{VoxelType, VoxelTypeId, voxel_components::VoxelComponent};

/// Complete properties for a voxel type
#[derive(Debug, Clone)]
pub struct VoxelProperties {
    pub id: VoxelTypeId,
    pub numeric_id: VoxelType,
    pub is_solid: bool,
    pub is_transparent: bool,
    pub hardness: f32,
    pub drops: Option<String>,
    pub components: Vec<Arc<dyn VoxelComponent>>,
}

impl VoxelProperties {
    /// Get a specific component by type
    pub fn get_component<T: 'static>(&self) -> Option<&T> {
        for component in &self.components {
            if let Some(comp) = component.as_any_ref().downcast_ref::<T>() {
                return Some(comp);
            }
        }
        None
    }

    /// Check if voxel has a specific component
    pub fn has_component<T: 'static>(&self) -> bool {
        self.get_component::<T>().is_some()
    }

    /// Get all components (for debugging)
    pub fn component_count(&self) -> usize {
        self.components.len()
    }
}
