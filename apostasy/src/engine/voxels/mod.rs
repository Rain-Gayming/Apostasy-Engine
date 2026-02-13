use serde::{Deserialize, Serialize};

use crate::engine::voxels::voxel_components::VoxelComponents;

pub mod voxel_components;
pub mod voxel_properties;
pub mod voxel_registry;

/// A VoxelTypeId is a unique identifier for a VoxelType
/// It is composed of a namespace, category, and name
/// Formatted as "namespace:category:name"
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct VoxelTypeId(pub String);

impl VoxelTypeId {
    /// Create a new VoxelTypeId
    pub fn new(namespace: &str, category: &str, name: &str) -> Self {
        Self(format!("{}:{}:{}", namespace, category, name))
    }

    /// Create a new VoxelTypeId from a string
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(id: &str) -> Self {
        Self(id.to_string())
    }

    /// Parse namespace, category, and name
    pub fn parse(&self) -> Option<(&str, &str, &str)> {
        let parts: Vec<&str> = self.0.split(':').collect();
        if parts.len() == 3 {
            Some((parts[0], parts[1], parts[2]))
        } else {
            None
        }
    }
}

/// Compact numeric ID for runtime use (assigned dynamically)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct VoxelType(pub u16);

impl VoxelType {
    // Air is always 0 defined by default
    pub const AIR: Self = Self(0);
}

/// YAML file format for voxel type definitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoxelDefinition {
    pub name: VoxelTypeId,

    #[serde(default = "default_true")]
    pub is_solid: bool,

    #[serde(default = "default_false")]
    pub is_transparent: bool,

    #[serde(default = "default_hardness")]
    pub hardness: f32,

    pub drops: Option<String>,

    #[serde(default)]
    pub components: VoxelComponents,
}

fn default_true() -> bool {
    true
}
fn default_false() -> bool {
    false
}
fn default_hardness() -> f32 {
    1.0
}
