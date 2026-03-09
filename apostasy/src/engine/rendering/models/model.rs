use std::collections::HashMap;

use crate as apostasy;
use crate::engine::assets::asset::Asset;
use crate::engine::assets::handle::Handle;
use crate::engine::editor::{EditorStorage, file_dragging_ui};
use crate::engine::rendering::models::material::MaterialAsset;

use crate::engine::editor::inspectable::{InspectValue, Inspectable};
use crate::engine::rendering::models::vertex::VertexType;
use apostasy_macros::{Component, SerializableComponent};
use ash::vk::{self};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug)]
pub struct GpuModel {
    pub meshes: Vec<GpuMesh>,
    pub name: String,
}

impl Asset for GpuModel {
    fn asset_type_name() -> &'static str {
        "GpuModel"
    }
}

#[derive(Debug, Clone, Default)]
pub struct GpuMesh {
    pub vertex_buffer: vk::Buffer,
    pub vertex_buffer_memory: vk::DeviceMemory,
    pub index_buffer: vk::Buffer,
    pub index_buffer_memory: vk::DeviceMemory,
    pub index_count: u32,
    pub vertex_type: VertexType,
    pub material_name: String,
}

#[derive(Component, Clone, Serialize, Deserialize, SerializableComponent)]
pub struct ModelRenderer {
    pub loaded_model: String,
    pub material_path: String,
    #[serde(skip)]
    pub material: Option<MaterialAsset>,
    #[serde(skip)]
    pub model_handle: Option<Handle<GpuModel>>,
    #[serde(skip)]
    pub material_handle: Option<Handle<MaterialAsset>>,
    #[serde(skip)]
    pub mesh_material_handles: HashMap<String, Handle<MaterialAsset>>,
}

impl Default for ModelRenderer {
    fn default() -> Self {
        Self {
            loaded_model: ".engine/cube.glb".to_string(),
            material_path: "".to_string(),
            material: None,
            model_handle: None,
            material_handle: None,
            mesh_material_handles: HashMap::new(),
        }
    }
}

impl Inspectable for ModelRenderer {
    fn inspect(&mut self, ui: &mut egui::Ui, editor_storage: &mut EditorStorage) -> bool {
        let mut remove = false;
        ui.horizontal(|ui| {
            if ui.small_button("✕").clicked() {
                remove = true;
            }
            egui::CollapsingHeader::new("ModelRenderer")
                .default_open(true)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("model:");
                        let (is_file, path) = file_dragging_ui(
                            ui,
                            editor_storage,
                            self.loaded_model.clone(),
                            ".glb".to_string(),
                            "Model".to_string(),
                        );

                        if is_file {
                            self.loaded_model = path;
                            self.model_handle = None;
                            editor_storage.file_dragging = false;
                            editor_storage.dragged_tree_node = None;
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("material:");

                        let material_name = if self.material_path.is_empty() {
                            "No material".to_string()
                        } else {
                            self.material_path.split(".").next().unwrap().to_string()
                        };

                        let (is_file, path) = file_dragging_ui(
                            ui,
                            editor_storage,
                            material_name,
                            ".material".to_string(),
                            "Material".to_string(),
                        );
                        if is_file {
                            self.material_path = path;
                            self.material_handle = None;
                            self.material = None;
                        }
                    });
                });
        });

        ui.separator();
        self.on_inspect();
        remove
    }
    fn on_inspect(&mut self) {}
}

impl InspectValue for ModelRenderer {
    fn inspect_value(&mut self, ui: &mut egui::Ui, editor_storage: &mut EditorStorage) {
        let _ = self.inspect(ui, editor_storage);
    }
}
