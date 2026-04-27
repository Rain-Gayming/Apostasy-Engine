use apostasy_macros::Component;

use crate::rendering::shared::model::GpuModel;

#[derive(Component, Default, Clone, Debug)]
pub struct ModelRenderer {
    pub model: Option<Box<GpuModel>>,
    pub model_path: String,
}

impl ModelRenderer {
    pub fn deserialize(&mut self, value: &serde_yaml::Value) -> anyhow::Result<()> {
        Ok(())
    }
    pub fn from_path(path: String) -> Self {
        let path = format!("{}{}", "res/", path);

        Self {
            model: None,
            model_path: path,
        }
    }
}
