use std::path::Path;

use crate::engine::assets::asset::{Asset, AssetLoadError, AssetLoader};
use crate::engine::assets::handle::Handle;
use crate::engine::assets::server::AssetServer;
use crate::engine::rendering::models::texture::GpuTexture;
use gltf::material::AlphaMode;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialAsset {
    pub name: String,
    pub base_color: [f32; 4],
    pub metallic: f32,
    pub roughness: f32,
    pub emissive: [f32; 3],
    #[serde(with = "alpha_mode_serde")]
    pub alpha_mode: AlphaMode,
    pub alpha_cutoff: f32,
    pub double_sided: bool,

    pub albedo_texture_name: Option<String>,
    pub metallic_texture_name: Option<String>,
    pub roughness_texture_name: Option<String>,
    pub normal_texture_name: Option<String>,
    pub emissive_texture_name: Option<String>,

    #[serde(skip)]
    pub textures_resolved: bool,
    #[serde(skip)]
    pub albedo_handle: Option<Handle<GpuTexture>>,
    #[serde(skip)]
    pub metallic_handle: Option<Handle<GpuTexture>>,
    #[serde(skip)]
    pub roughness_handle: Option<Handle<GpuTexture>>,
    #[serde(skip)]
    pub normal_handle: Option<Handle<GpuTexture>>,
    #[serde(skip)]
    pub emissive_handle: Option<Handle<GpuTexture>>,
}

impl Asset for MaterialAsset {
    fn asset_type_name() -> &'static str {
        "MaterialAsset"
    }
}

impl Default for MaterialAsset {
    fn default() -> Self {
        Self {
            name: "material".to_string(),
            base_color: [1.0, 1.0, 1.0, 1.0],
            metallic: 0.0,
            roughness: 1.0,
            emissive: [0.0, 0.0, 0.0],
            alpha_mode: AlphaMode::Opaque,
            alpha_cutoff: 0.5,
            double_sided: false,
            albedo_texture_name: Some("temp.png".to_string()),
            metallic_texture_name: None,
            roughness_texture_name: None,
            normal_texture_name: None,
            emissive_texture_name: None,
            textures_resolved: false,
            albedo_handle: None,
            metallic_handle: None,
            roughness_handle: None,
            normal_handle: None,
            emissive_handle: None,
        }
    }
}

impl MaterialAsset {
    pub fn resolve_textures(&mut self, server: &crate::engine::assets::server::AssetServer) {
        self.albedo_handle = resolve_one(&self.albedo_texture_name, server);
        self.metallic_handle = resolve_one(&self.metallic_texture_name, server);
        self.roughness_handle = resolve_one(&self.roughness_texture_name, server);
        self.normal_handle = resolve_one(&self.normal_texture_name, server);
        self.emissive_handle = resolve_one(&self.emissive_texture_name, server);
        self.textures_resolved = true;
    }

    pub fn has_albedo(&self) -> bool {
        self.albedo_handle.is_some()
    }

    pub fn save(&self) {
        let path = format!("{}{}.yaml", "res/assets/materials", self.name);
        match serde_yaml::to_string(self) {
            Ok(yaml) => {
                if let Err(e) = std::fs::write(&path, yaml) {
                    eprintln!("[MaterialAsset] Failed to write '{}': {}", path, e);
                }
            }
            Err(e) => eprintln!("[MaterialAsset] Serialize error: {}", e),
        }
    }
}

fn resolve_one(
    name: &Option<String>,
    server: &crate::engine::assets::server::AssetServer,
) -> Option<Handle<GpuTexture>> {
    let name = name.as_ref()?;
    match server.load_cached::<GpuTexture>(name) {
        Ok(h) => Some(h),
        Err(e) => {
            eprintln!("[MaterialAsset] Could not load texture '{}': {}", name, e);
            None
        }
    }
}

pub struct MaterialLoader;

impl AssetLoader for MaterialLoader {
    type Asset = MaterialAsset;

    fn extensions(&self) -> &[&str] {
        &["yaml", "yml"]
    }

    fn load_sync(&self, path: &Path) -> Result<MaterialAsset, AssetLoadError> {
        let src = std::fs::read_to_string(path).map_err(|e| AssetLoadError::Io {
            path: path.display().to_string(),
            source: e,
        })?;

        let mat: MaterialAsset = serde_yaml::from_str(&src).map_err(|e| AssetLoadError::Parse {
            path: path.display().to_string(),
            message: e.to_string(),
        })?;

        Ok(mat)
    }
}

mod alpha_mode_serde {
    use gltf::material::AlphaMode;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(mode: &AlphaMode, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(match mode {
            AlphaMode::Opaque => "OPAQUE",
            AlphaMode::Mask => "MASK",
            AlphaMode::Blend => "BLEND",
        })
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<AlphaMode, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer).unwrap_or_else(|_| "OPAQUE".to_string());
        Ok(match s.as_str() {
            "MASK" => AlphaMode::Mask,
            "BLEND" => AlphaMode::Blend,
            _ => AlphaMode::Opaque,
        })
    }
}
