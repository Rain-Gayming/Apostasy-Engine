use ash::vk;

/// The settings a pipeline/renderer can have
#[derive(Default, Clone, Copy, PartialEq)]
pub struct PipelineSettings {
    pub depth_settings: DepthSettings,
    pub rasterizeation_settings: RasterizationSettings,
    pub image_settings: ImageSettings,
}

/// The settings for a depth test
#[derive(Clone, Copy, PartialEq)]
pub struct DepthSettings {
    pub depth_test_enabled: bool,
    pub depth_compare_op: vk::CompareOp,
}

impl Default for DepthSettings {
    fn default() -> Self {
        Self {
            depth_test_enabled: true,
            depth_compare_op: vk::CompareOp::LESS,
        }
    }
}

/// The settings for rasterization
#[derive(Clone, Copy, PartialEq)]
pub struct RasterizationSettings {
    pub polygon_mode: vk::PolygonMode,
    pub cull_mode: vk::CullModeFlags,
    pub front_face: vk::FrontFace,
    pub line_width: f32,
}

impl Default for RasterizationSettings {
    fn default() -> Self {
        Self {
            polygon_mode: vk::PolygonMode::FILL,
            cull_mode: vk::CullModeFlags::BACK,
            front_face: vk::FrontFace::COUNTER_CLOCKWISE,
            line_width: 1.0,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct ImageSettings {
    pub filter_mode: vk::Filter,
    pub address_mode: vk::SamplerAddressMode,
    pub anisotropy_enabled: bool,
    pub anisotropy_amount: u8,
    pub mip_map_mode: vk::SamplerMipmapMode,
}

impl Default for ImageSettings {
    fn default() -> Self {
        Self {
            filter_mode: vk::Filter::NEAREST,
            address_mode: vk::SamplerAddressMode::REPEAT,
            anisotropy_enabled: false,
            anisotropy_amount: 16,
            mip_map_mode: vk::SamplerMipmapMode::LINEAR,
        }
    }
}
