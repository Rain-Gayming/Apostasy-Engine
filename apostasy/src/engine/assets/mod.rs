use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum AssetType {
    Image,
    Audio,
    Scene,
    Shader,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AssetPath {
    pub path: String,
    pub name: String,
    pub extension: String,
    pub asset_type: AssetType,
}

impl AssetPath {
    pub fn new(path: String, name: String, extension: String, asset_type: AssetType) -> Self {
        Self {
            path,
            name,
            extension,
            asset_type,
        }
    }
}

pub const ASSET_DIR: &str = "res/";
pub const ENGINE_SETTINGS_LOCATION: &str = "settings/";
