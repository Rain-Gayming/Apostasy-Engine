use ash::vk;
use serde::{Deserialize, Serialize};

macro_rules! vk_str_serde {
    ($mod_name:ident, $Type:ty, { $($variant:ident => $name:literal),* $(,)? }) => {
        mod $mod_name {
            use ash::vk;
            use serde::{Deserialize, Deserializer, Serializer};
            use serde::de::{self, Unexpected};

            pub fn serialize<S: Serializer>(v: &$Type, s: S) -> Result<S::Ok, S::Error> {
                match *v {
                    $( <$Type>::$variant => s.serialize_str($name), )*
                    other => s.serialize_i32(other.as_raw()),
                }
            }

            pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<$Type, D::Error> {
                let raw = String::deserialize(d)?;
                match raw.as_str() {
                    $( $name => Ok(<$Type>::$variant), )*
                    other => Err(de::Error::invalid_value(
                        Unexpected::Str(other),
                        &concat!("a valid ", stringify!($Type), " name"),
                    )),
                }
            }
        }
    };
}

vk_str_serde!(serde_compare_op, vk::CompareOp, {
    NEVER            => "NEVER",
    LESS             => "LESS",
    EQUAL            => "EQUAL",
    LESS_OR_EQUAL    => "LESS_OR_EQUAL",
    GREATER          => "GREATER",
    NOT_EQUAL        => "NOT_EQUAL",
    GREATER_OR_EQUAL => "GREATER_OR_EQUAL",
    ALWAYS           => "ALWAYS",
});

vk_str_serde!(serde_polygon_mode, vk::PolygonMode, {
    FILL              => "FILL",
    LINE              => "LINE",
    POINT             => "POINT",
    FILL_RECTANGLE_NV => "FILL_RECTANGLE_NV",
});

// CullModeFlags is a bitmask (u32), not an i32 enum — handled separately
mod serde_cull_mode {
    use ash::vk;
    use serde::de::{self, Unexpected};
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(v: &vk::CullModeFlags, s: S) -> Result<S::Ok, S::Error> {
        let name = match *v {
            vk::CullModeFlags::NONE => "NONE",
            vk::CullModeFlags::FRONT => "FRONT",
            vk::CullModeFlags::BACK => "BACK",
            vk::CullModeFlags::FRONT_AND_BACK => "FRONT_AND_BACK",
            other => return s.serialize_u32(other.as_raw()),
        };
        s.serialize_str(name)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<vk::CullModeFlags, D::Error> {
        let raw = String::deserialize(d)?;
        match raw.as_str() {
            "NONE" => Ok(vk::CullModeFlags::NONE),
            "FRONT" => Ok(vk::CullModeFlags::FRONT),
            "BACK" => Ok(vk::CullModeFlags::BACK),
            "FRONT_AND_BACK" => Ok(vk::CullModeFlags::FRONT_AND_BACK),
            other => Err(de::Error::invalid_value(
                Unexpected::Str(other),
                &"a valid VkCullModeFlags name",
            )),
        }
    }
}

vk_str_serde!(serde_front_face, vk::FrontFace, {
    COUNTER_CLOCKWISE => "COUNTER_CLOCKWISE",
    CLOCKWISE         => "CLOCKWISE",
});

vk_str_serde!(serde_filter, vk::Filter, {
    NEAREST  => "NEAREST",
    LINEAR   => "LINEAR",
    CUBIC_EXT => "CUBIC_EXT",
});

vk_str_serde!(serde_sampler_address_mode, vk::SamplerAddressMode, {
    REPEAT               => "REPEAT",
    MIRRORED_REPEAT      => "MIRRORED_REPEAT",
    CLAMP_TO_EDGE        => "CLAMP_TO_EDGE",
    CLAMP_TO_BORDER      => "CLAMP_TO_BORDER",
    MIRROR_CLAMP_TO_EDGE => "MIRROR_CLAMP_TO_EDGE",
});

vk_str_serde!(serde_mipmap_mode, vk::SamplerMipmapMode, {
    NEAREST => "NEAREST",
    LINEAR  => "LINEAR",
});

/// The settings a pipeline/renderer can have
#[derive(Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PipelineSettings {
    pub depth_settings: DepthSettings,
    pub rasterization_settings: RasterizationSettings,
    pub image_settings: ImageSettings,
    pub debug_settings: DebugSettings,
}

impl Default for PipelineSettings {
    fn default() -> Self {
        Self {
            depth_settings: DepthSettings::default(),
            rasterization_settings: RasterizationSettings::default(),
            image_settings: ImageSettings::default(),
            debug_settings: DebugSettings::default(),
        }
    }
}

/// The settings for a depth test
#[derive(Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DepthSettings {
    pub depth_test_enabled: bool,
    #[serde(with = "serde_compare_op")]
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
#[derive(Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RasterizationSettings {
    #[serde(with = "serde_polygon_mode")]
    pub polygon_mode: vk::PolygonMode,
    #[serde(with = "serde_cull_mode")]
    pub cull_mode: vk::CullModeFlags,
    #[serde(with = "serde_front_face")]
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

/// The settings for image sampling
#[derive(Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ImageSettings {
    #[serde(with = "serde_filter")]
    pub filter_mode: vk::Filter,
    #[serde(with = "serde_sampler_address_mode")]
    pub address_mode: vk::SamplerAddressMode,
    pub anisotropy_enabled: bool,
    pub anisotropy_amount: u8,
    #[serde(with = "serde_mipmap_mode")]
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

#[derive(Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DebugSettings {
    pub debug_line_width: f32,
}

impl Default for DebugSettings {
    fn default() -> Self {
        Self {
            debug_line_width: 1.0,
        }
    }
}
