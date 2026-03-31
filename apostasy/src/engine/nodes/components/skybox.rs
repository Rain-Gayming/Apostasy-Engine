use crate::engine::assets::handle::Handle;
use crate::engine::editor::{EditorStorage, file_manager::file_dragging_ui};
use crate::engine::rendering::models::{model::GpuModel, texture::GpuTexture};
use crate::{self as apostasy};
use apostasy_macros::{Component, SerializableComponent};
use serde::{Deserialize, Serialize};

#[derive(Component, Clone, Serialize, Deserialize, SerializableComponent)]
/// Adds a skybox, doesn't require a transform
/// only add 1
pub struct Skybox {
    pub texture_path: String,
    #[serde(skip)]
    pub texture_handle: Option<Handle<GpuTexture>>,
    #[serde(skip)]
    pub cube_model_handle: Option<Handle<GpuModel>>,
}

impl Default for Skybox {
    fn default() -> Self {
        Self {
            texture_path: ".engine/missing-texture.png".to_string(),
            texture_handle: None,
            cube_model_handle: None,
        }
    }
}

impl crate::engine::editor::inspectable::Inspectable for Skybox {
    fn inspect(&mut self, ui: &mut egui::Ui, editor_storage: &mut EditorStorage) -> bool {
        ui.horizontal(|ui| {
            ui.label("Skybox texture:");
            let (is_file, path) = file_dragging_ui(
                ui,
                editor_storage,
                self.texture_path.clone(),
                ".png".to_string(),
                "Texture".to_string(),
            );

            if is_file {
                self.texture_path = path;
                self.texture_handle = None;
                editor_storage.file_dragging = false;
                editor_storage.dragged_tree_node = None;
            }
        });
        false
    }

    fn on_inspect(&mut self) {}
}
