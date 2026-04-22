use std::sync::{Arc, RwLock};

use anyhow::{Error, Result};

use crate::{
    assets::loader::AssetLoader,
    log_warn,
    objects::component::{BoxedComponent, get_component_registration},
    voxels::{
        texture_atlas::AtlasBuilder,
        voxel::{VoxelDefinition, VoxelId, VoxelRegistry, VoxelTextures},
    },
};

pub struct VoxelLoader {
    pub registry: Arc<RwLock<VoxelRegistry>>,
    pub atlas_builder: Arc<RwLock<AtlasBuilder>>,
}

impl AssetLoader for VoxelLoader {
    fn class_name(&self) -> &'static str {
        "Voxel"
    }

    fn load(&self, raw: &serde_yaml::Value) -> Result<()> {
        let name: String = raw["name"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing 'name'"))?
            .to_string();

        let namespace: String = raw["namespace"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing 'namespace'"))?
            .to_string();

        {
            let registry = self.registry.read().unwrap();

            for reg in registry.defs.iter() {
                if reg.name == name && reg.namespace == namespace {
                    let msg = format!(
                        "Voxel with the name: {} exists in name space {} already",
                        name.to_string(),
                        namespace.to_string()
                    );

                    return Err(Error::msg(msg));
                }
            }
        }

        let textures = if let Some(tex) = raw["textures"].as_mapping() {
            let mut atlas = self.atlas_builder.write().unwrap();

            let mut get = |key: &str| -> u32 {
                tex.get(key)
                    .or_else(|| tex.get("all"))
                    .and_then(|v| v.as_str())
                    .map(|path| atlas.add_texture(path))
                    .unwrap_or(0)
            };

            VoxelTextures {
                top: get("top"),
                bottom: get("bottom"),
                front: get("front"),
                back: get("back"),
                left: get("left"),
                right: get("right"),
            }
        } else {
            VoxelTextures::all(0)
        };

        let mut components: Vec<BoxedComponent> = Vec::new();

        if let Some(comp_map) = raw["components"].as_mapping() {
            for (key, value) in comp_map {
                let component_name = key
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("Invalid component key"))?;

                if let Some(registration) = get_component_registration(component_name) {
                    let mut component = (registration.create)();
                    component.deserialize(value)?;
                    components.push(component);
                } else {
                    log_warn!("Unknown component: {}", component_name);
                }
            }
        }

        let def = VoxelDefinition {
            name: name.clone(),
            namespace: namespace.clone(),
            class: "Voxel".to_string(),
            components,
            textures,
        };

        let mut registry = self.registry.write().unwrap();

        for reg in registry.defs.iter() {
            if reg.name == name && reg.namespace == namespace {
                let msg = format!(
                    "Voxel with the name: {} exists in name space {} already",
                    name.to_string(),
                    namespace.to_string()
                );

                return Err(Error::msg(msg));
            }
        }

        let id = registry.defs.len() as VoxelId;
        let full_name = format!("{}:{}", namespace, name);
        registry.defs.push(def);
        registry.name_to_id.insert(full_name.clone(), id);
        registry.id_to_name.insert(id, full_name);

        Ok(())
    }
}
