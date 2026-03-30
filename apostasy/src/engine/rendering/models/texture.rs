use std::{path::Path, sync::Arc};

use anyhow::Error;
use ash::vk;

use crate::engine::{
    assets::asset::{Asset, AssetLoadError, AssetLoader},
    rendering::{pipeline_settings::PipelineSettings, rendering_context::RenderingContext},
};

#[derive(Clone, Debug)]
pub struct GpuTexture {
    pub name: String,
    pub image: vk::Image,
    pub image_view: vk::ImageView,
    pub memory: vk::DeviceMemory,
    pub sampler: vk::Sampler,
    pub descriptor_set: vk::DescriptorSet,
}

impl Asset for GpuTexture {
    fn asset_type_name() -> &'static str {
        "GpuTexture"
    }
}

pub struct GpuTextureLoader {
    pub context: Arc<RenderingContext>,
    pub command_pool: vk::CommandPool,
    pub descriptor_pool: vk::DescriptorPool,
    pub descriptor_set_layout: vk::DescriptorSetLayout,
    pub default_ubo: vk::Buffer,
    pub settings: PipelineSettings,
}

impl GpuTextureLoader {
    pub fn new(
        context: Arc<RenderingContext>,
        command_pool: vk::CommandPool,
        descriptor_pool: vk::DescriptorPool,
        descriptor_set_layout: vk::DescriptorSetLayout,
        default_ubo: vk::Buffer,
        settings: PipelineSettings,
    ) -> Self {
        Self {
            context,
            command_pool,
            descriptor_pool,
            descriptor_set_layout,
            default_ubo,
            settings,
        }
    }
}

impl AssetLoader for GpuTextureLoader {
    type Asset = GpuTexture;

    fn extensions(&self) -> &[&str] {
        &["png", "jpg", "jpeg", "bmp", "tga", "webp"]
    }

    fn load_sync(&self, path: &Path) -> Result<GpuTexture, AssetLoadError> {
        let name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("texture")
            .to_string();

        println!("path: {}", path.display());

        let path_str = path
            .to_str()
            .ok_or_else(|| AssetLoadError::other("Non-UTF-8 path"))?;

        let old_tex = self
            .context
            .load_texture(
                path_str,
                self.command_pool,
                self.descriptor_pool,
                self.descriptor_set_layout,
                self.default_ubo,
                self.settings,
            )
            .map_err(|e: Error| AssetLoadError::other(e.to_string()))?;

        // Convert from the old model::Texture to GpuTexture.
        // The fields are identical — this is a zero-cost rename.
        Ok(GpuTexture {
            name,
            image: old_tex.image,
            image_view: old_tex.image_view,
            memory: old_tex.memory,
            sampler: old_tex.sampler,
            descriptor_set: old_tex.descriptor_set,
        })
    }
}
