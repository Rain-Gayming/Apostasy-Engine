use crate::{log, log_error};
use std::{path::Path, sync::Arc};

use apostasy_macros::Resource;
use rustc_hash::FxHashMap;

use crate::{
    self as apostasy,
    engine::voxels::{
        VoxelDefinition, VoxelType, VoxelTypeId,
        voxel_components::{VoxelComponent, VoxelComponents},
        voxel_properties::VoxelProperties,
    },
};

/// Global voxel type registry
#[derive(Resource, Clone)]
pub struct VoxelRegistry {
    // Numeric ID -> Properties
    by_numeric_id: Arc<FxHashMap<VoxelType, VoxelProperties>>,

    // String ID -> Numeric ID mapping
    id_mapping: Arc<FxHashMap<VoxelTypeId, VoxelType>>,

    // Counter for assigning numeric IDs
    next_id: u16,
}

impl Default for VoxelRegistry {
    fn default() -> Self {
        let mut registry = Self {
            by_numeric_id: Arc::new(FxHashMap::default()),
            id_mapping: Arc::new(FxHashMap::default()),
            next_id: 1, // 0 is reserved for AIR
        };

        // Register AIR
        registry.register_voxel(VoxelDefinition {
            name: VoxelTypeId::from_str("apostasy:voxel:air"),
            is_solid: false,
            is_transparent: true,
            hardness: 0.0,
            drops: None,
            components: VoxelComponents::default(),
        });

        registry
    }
}

impl VoxelRegistry {
    /// Register a voxel type from definition
    pub fn register_voxel(&mut self, definition: VoxelDefinition) -> VoxelType {
        // Check if already registered
        if let Some(&existing_id) = self.id_mapping.get(&definition.name) {
            return existing_id;
        }

        // Assign numeric ID
        let numeric_id = if definition.name.0 == "apostasy:voxel:air" {
            VoxelType::AIR
        } else {
            let id = VoxelType(self.next_id);
            self.next_id += 1;
            id
        };

        log!("Registering voxel: {}", definition.name.0);

        // Build components list
        let mut components: Vec<Arc<dyn VoxelComponent>> = Vec::new();

        if let Some(trans) = definition.components.transitionable {
            log!("Voxel: {} is transitionable", definition.name.0);
            components.push(Arc::new(trans));
        }

        // Create properties
        let properties = VoxelProperties {
            id: definition.name.clone(),
            numeric_id,
            is_solid: definition.is_solid,
            is_transparent: definition.is_transparent,
            hardness: definition.hardness,
            drops: definition.drops,
            components,
        };

        // Store in maps
        Arc::make_mut(&mut self.by_numeric_id).insert(numeric_id, properties);
        Arc::make_mut(&mut self.id_mapping).insert(definition.name, numeric_id);

        numeric_id
    }

    /// Get properties by numeric ID
    pub fn get(&self, voxel_type: VoxelType) -> Option<&VoxelProperties> {
        self.by_numeric_id.get(&voxel_type)
    }

    /// Get properties by string ID
    pub fn get_by_id(&self, id: &VoxelTypeId) -> Option<&VoxelProperties> {
        self.id_mapping
            .get(id)
            .and_then(|numeric_id| self.by_numeric_id.get(numeric_id))
    }

    /// Get numeric ID from string ID
    pub fn get_numeric_id(&self, id: &VoxelTypeId) -> Option<VoxelType> {
        self.id_mapping.get(id).copied()
    }

    /// Load from a single YAML file
    pub fn load_from_yaml(&mut self, path: impl AsRef<Path>) -> Result<VoxelType, String> {
        let content = std::fs::read_to_string(path.as_ref())
            .map_err(|e| format!("Failed to read file: {}", e))?;

        let definition: VoxelDefinition =
            serde_yaml::from_str(&content).map_err(|e| format!("Failed to parse YAML: {}", e))?;

        Ok(self.register_voxel(definition))
    }

    /// Load all YAML files from a directory
    pub fn load_from_directory(&mut self, dir: impl AsRef<Path>) -> Result<Vec<VoxelType>, String> {
        let mut loaded = Vec::new();

        let entries = std::fs::read_dir(dir.as_ref())
            .map_err(|e| format!("Failed to read directory: {}", e))?;

        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("yaml")
                || path.extension().and_then(|s| s.to_str()) == Some("yml")
            {
                match self.load_from_yaml(&path) {
                    Ok(id) => loaded.push(id),
                    Err(e) => log_error!("Warning: Failed to load {:?}: {}", path, e),
                }
            }
        }

        Ok(loaded)
    }

    /// Get all registered voxel types
    pub fn all_voxels(&self) -> impl Iterator<Item = (&VoxelType, &VoxelProperties)> {
        self.by_numeric_id.iter()
    }
}
