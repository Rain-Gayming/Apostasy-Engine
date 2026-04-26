use std::sync::{Arc, RwLock};

use anyhow::{Error, Result};

use crate::{
    assets::loader::AssetLoader,
    voxels::biome::{BiomeDefinition, BiomeRegistry},
};

pub struct BiomeLoader {
    pub registry: Arc<RwLock<BiomeRegistry>>,
}

impl AssetLoader for BiomeLoader {
    fn class_name(&self) -> &'static str {
        "Biome"
    }

    fn load(&mut self, raw: &serde_yaml::Value) -> Result<()> {
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
                        "Biome with the name: {} exists in name space {} already",
                        name.to_string(),
                        namespace.to_string()
                    );

                    return Err(Error::msg(msg));
                }
            }
        }

        let surface_voxels = raw["voxel"]["surface"]
            .as_sequence()
            .ok_or_else(|| anyhow::anyhow!("Missing 'voxel.surface'"))?
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect::<Vec<_>>();

        let subsurface_voxels = raw["voxel"]["subsurface"]
            .as_sequence()
            .ok_or_else(|| anyhow::anyhow!("Missing 'voxel.subsurface'"))?
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect::<Vec<_>>();

        let amplitude = raw["amplitude"]
            .as_f64()
            .ok_or_else(|| anyhow::anyhow!("Missing 'amplitude'"))?;

        let frequency = raw["frequency"]
            .as_f64()
            .ok_or_else(|| anyhow::anyhow!("Missing 'frequency'"))?;

        let humidity = raw["humidity"]
            .as_f64()
            .ok_or_else(|| anyhow::anyhow!("Missing 'humidity'"))?;

        let temperature = raw["temperature"]
            .as_f64()
            .ok_or_else(|| anyhow::anyhow!("Missing 'temperature'"))?;
        let def = BiomeDefinition {
            name: name.clone(),
            namespace: namespace.clone(),
            class: "Biome".to_string(),

            surface_voxels,
            subsurface_voxels,

            amplitude,
            frequency,

            humidity,
            temperature,
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

        let id = registry.defs.len() as u16;
        let full_name = format!("{}:Biomes:{}", namespace, name);
        registry.defs.push(def);

        registry.name_to_id.insert(full_name.clone(), id);
        registry.id_to_name.insert(id, full_name);
        Ok(())
    }
}
