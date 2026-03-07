use ash::{Device, vk};
use std::path::Path;

use crate::engine::assets::asset::{Asset, AssetLoadError, AssetLoader};

#[derive(Clone)]
pub struct ShaderSpirv {
    pub name: String,
    pub bytes: Vec<u8>,
}

impl Asset for ShaderSpirv {
    fn asset_type_name() -> &'static str {
        "ShaderSpirv"
    }
}

impl ShaderSpirv {
    pub fn create_module(&self, device: &Device) -> Result<vk::ShaderModule, vk::Result> {
        unsafe {
            let code_u32 = self.as_u32_words();
            let create_info = vk::ShaderModuleCreateInfo::default().code(&code_u32);
            device.create_shader_module(&create_info, None)
        }
    }

    fn as_u32_words(&self) -> Vec<u32> {
        assert!(
            self.bytes.len() % 4 == 0,
            "SPIR-V byte length must be a multiple of 4"
        );
        self.bytes
            .chunks_exact(4)
            .map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]]))
            .collect()
    }
}

pub struct ShaderLoader;

impl AssetLoader for ShaderLoader {
    type Asset = ShaderSpirv;

    fn extensions(&self) -> &[&str] {
        &["spv"]
    }

    fn load_sync(&self, path: &Path) -> Result<ShaderSpirv, AssetLoadError> {
        let bytes = std::fs::read(path).map_err(|e| AssetLoadError::Io {
            path: path.display().to_string(),
            source: e,
        })?;

        println!("Loading shader: {}", path.display());

        if bytes.len() < 4 || &bytes[0..4] != b"\x03\x02\x23\x07" {
            return Err(AssetLoadError::Parse {
                path: path.display().to_string(),
                message: "Not a valid SPIR-V file (bad magic number)".into(),
            });
        }

        Ok(ShaderSpirv {
            name: path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("shader")
                .to_string(),
            bytes,
        })
    }
}
