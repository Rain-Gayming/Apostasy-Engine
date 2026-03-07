use std::collections::HashMap;

use crate as apostasy;
use crate::engine::assets::asset::Asset;
use crate::engine::assets::handle::Handle;
use crate::engine::assets::server::AssetServer;
use crate::engine::editor::EditorStorage;
use crate::engine::rendering::models::material::MaterialAsset;

use crate::engine::editor::inspectable::{InspectValue, Inspectable};
use crate::engine::rendering::models::vertex::VertexType;
use apostasy_macros::{Component, SerializableComponent};
use ash::vk::{self};
use egui::{Button, Label, Sense, Vec2};
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
    pub material: Option<MaterialAsset>,
    #[serde(skip)]
    pub model_handle: Option<Handle<GpuModel>>,
    #[serde(skip)]
    pub material_handle: Option<Handle<MaterialAsset>>,
    pub mesh_material_handles: HashMap<String, Handle<MaterialAsset>>,
}

impl Default for ModelRenderer {
    fn default() -> Self {
        Self {
            loaded_model: "cube.glb".to_string(),
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
                        let response = ui.add(
                            Button::new(self.loaded_model.clone())
                                .sense(Sense::drag())
                                .sense(Sense::hover())
                                .min_size(Vec2::new(100.0, 25.0)),
                        );

                        if response.contains_pointer() {
                            if let Some(tree_node) = &editor_storage.selected_tree_node {
                                if tree_node.ends_with(".glb") {
                                    egui::Tooltip::always_open(
                                        ui.ctx().clone(),
                                        ui.layer_id(),
                                        egui::Id::new("file_drag_tooltip_2"),
                                        response.rect,
                                    )
                                    .at_pointer()
                                    .show(|ui| {
                                        ui.label("set model");
                                    });
                                } else {
                                    egui::Tooltip::always_open(
                                        ui.ctx().clone(),
                                        ui.layer_id(),
                                        egui::Id::new("drag_hint"),
                                        response.rect,
                                    )
                                    .at_pointer()
                                    .show(|ui| {
                                        ui.label("Drag any .glb file here");
                                    });
                                }
                            }
                        }

                        if response.hovered() {
                            if let Some(tree_node) = &editor_storage.selected_tree_node {
                                if tree_node.ends_with(".glb") {
                                    let mut path = tree_node.to_string();
                                    // split off after "res/"
                                    let path = path.split_off(4);
                                    println!("path: {}", path);

                                    self.loaded_model = path;
                                    self.model_handle = None;

                                    editor_storage.file_dragging = false;
                                }
                            } else {
                                egui::Tooltip::always_open(
                                    ui.ctx().clone(),
                                    ui.layer_id(),
                                    egui::Id::new("drag_hint"),
                                    response.rect,
                                )
                                .at_pointer()
                                .show(|ui| {
                                    ui.label("Drag any .glb file here");
                                });
                            }
                        }
                    });

                    // ui.horizontal(|ui| {
                    //     ui.label("material:");
                    //     match &mut self.material {
                    //         Some(mat) => {
                    //             if ui.small_button("Remove").clicked() {
                    //                 self.material = None;
                    //             } else {
                    //                 egui::CollapsingHeader::new("Material")
                    //                     .default_open(false)
                    //                     .show(ui, |ui| {
                    //                         let _ = mat.inspect(ui);
                    //                     });
                    //             }
                    //         }
                    //         None => {
                    //             if ui.small_button("Add Material").clicked() {
                    //                 self.material = Some(Material::default());
                    //             }
                    //         }
                    //     }
                    // });
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

// impl Material {
//     pub fn albedo_texture(&mut self) -> &mut Option<Texture> {
//         &mut self.albedo_color_texture
//     }
//
//     pub fn metallic_texture(&mut self) -> &mut Option<Texture> {
//         &mut self.metallic_texture
//     }
//
//     pub fn roughness_texture(&mut self) -> &mut Option<Texture> {
//         &mut self.roughness_texture
//     }
//
//     pub fn normal_texture(&mut self) -> &mut Option<Texture> {
//         &mut self.normal_texture
//     }
//
//     pub fn emissive_texture(&mut self) -> &mut Option<Texture> {
//         &mut self.emissive_texture
//     }
//
//     pub fn set_albedo_texture(&mut self, texture: Texture) {
//         self.albedo_color_texture = Some(texture);
//     }
//
//     pub fn load(name: &str) -> Option<Material> {
//         let path = ASSET_DIR.to_string() + name + ".yaml";
//         if !Path::new(&path).exists() {
//             return None;
//         }
//
//         let contents = std::fs::read_to_string(path).ok()?;
//         #[derive(serde::Deserialize)]
//         struct MaterialDef {
//             name: Option<String>,
//             base_color: Option<[f32; 4]>,
//             metallic: Option<f32>,
//             roughness: Option<f32>,
//             emissive: Option<[f32; 3]>,
//             alpha_mode: Option<String>,
//             alpha_cutoff: Option<f32>,
//             double_sided: Option<bool>,
//             albedo_texture: Option<Option<String>>,
//             metallic_roughness_texture: Option<Option<String>>,
//             normal_texture: Option<Option<String>>,
//             emissive_texture: Option<Option<String>>,
//         }
//
//         let def: MaterialDef = serde_yaml::from_str(&contents).ok()?;
//
//         let alpha_mode = match def.alpha_mode.as_deref() {
//             Some("MASK") => AlphaMode::Mask,
//             Some("BLEND") => AlphaMode::Blend,
//             _ => AlphaMode::Opaque,
//         };
//
//         Some(Material {
//             name: def.name.unwrap_or_else(|| name.to_string()),
//             base_color: def.base_color.unwrap_or([1.0, 1.0, 1.0, 1.0]),
//             metallic: def.metallic.unwrap_or(0.0),
//             roughness: def.roughness.unwrap_or(1.0),
//             emissive: def.emissive.unwrap_or([0.0, 0.0, 0.0]),
//             alpha_mode,
//             alpha_cutoff: def.alpha_cutoff.unwrap_or(0.5),
//             double_sided: def.double_sided.unwrap_or(false),
//             albedo_texture_name: def.albedo_texture.and_then(|v| v),
//             albedo_color_texture: None,
//             albedo_texture_loaded_name: None,
//             metallic_texture_name: def.metallic_roughness_texture.and_then(|v| v),
//             metallic_texture: None,
//             roughness_texture_name: None,
//             roughness_texture: None,
//             normal_texture_name: def.normal_texture.and_then(|v| v),
//             normal_texture: None,
//             emmisive_texture_name: def.emissive_texture.and_then(|v| v),
//             emissive_texture: None,
//         })
//     }
//
//     pub fn take_albedo_texture(&mut self) -> Option<Texture> {
//         self.albedo_color_texture.take()
//     }
//
//     pub fn serialize_material(&self) {
//         let mut output = serde_yaml::Mapping::new();
//         output.insert(
//             serde_yaml::Value::String("name".into()),
//             serde_yaml::to_value(self.name.clone()).unwrap(),
//         );
//         output.insert(
//             serde_yaml::Value::String("base_color".into()),
//             serde_yaml::to_value(self.base_color).unwrap(),
//         );
//         output.insert(
//             serde_yaml::Value::String("metallic".into()),
//             serde_yaml::to_value(self.metallic).unwrap(),
//         );
//         output.insert(
//             serde_yaml::Value::String("roughness".into()),
//             serde_yaml::to_value(self.roughness).unwrap(),
//         );
//         output.insert(
//             serde_yaml::Value::String("emissive".into()),
//             serde_yaml::to_value(self.emissive).unwrap(),
//         );
//         output.insert(
//             serde_yaml::Value::String("alpha_mode".into()),
//             serde_yaml::to_value(self.alpha_mode).unwrap(),
//         );
//         output.insert(
//             serde_yaml::Value::String("alpha_cutoff".into()),
//             serde_yaml::to_value(self.alpha_cutoff).unwrap(),
//         );
//         output.insert(
//             serde_yaml::Value::String("double_sided".into()),
//             serde_yaml::to_value(self.double_sided).unwrap(),
//         );
//         output.insert(
//             serde_yaml::Value::String("albedo_texture".into()),
//             serde_yaml::to_value(self.albedo_texture_name.clone()).unwrap(),
//         );
//         output.insert(
//             serde_yaml::Value::String("metallic_roughness_texture".into()),
//             serde_yaml::to_value(self.metallic_texture_name.clone()).unwrap(),
//         );
//         output.insert(
//             serde_yaml::Value::String("normal_texture".into()),
//             serde_yaml::to_value(self.normal_texture_name.clone()).unwrap(),
//         );
//         output.insert(
//             serde_yaml::Value::String("emissive_texture".into()),
//             serde_yaml::to_value(self.emmisive_texture_name.clone()).unwrap(),
//         );
//
//         let path = ASSET_DIR.to_string() + &self.name + ".yaml";
//         if !Path::new(&path).exists() {
//             std::fs::create_dir_all(ASSET_DIR).unwrap();
//         }
//         std::fs::write(path, serde_yaml::to_string(&output).unwrap()).unwrap();
//     }
// }
//
// impl Inspectable for Material {
//     fn inspect(&mut self, ui: &mut egui::Ui) -> bool {
//         let mut changed = false;
//
//         ui.horizontal(|ui| {
//             ui.label("Name:");
//             if ui.text_edit_singleline(&mut self.name).changed() {
//                 changed = true;
//             }
//         });
//
//         ui.horizontal(|ui| {
//             ui.label("Base Color:");
//             let mut r = self.base_color[0] as f64;
//             let mut g = self.base_color[1] as f64;
//             let mut b = self.base_color[2] as f64;
//             let mut a = self.base_color[3] as f64;
//             ui.add(egui::DragValue::new(&mut r).speed(0.01));
//             ui.add(egui::DragValue::new(&mut g).speed(0.01));
//             ui.add(egui::DragValue::new(&mut b).speed(0.01));
//             ui.add(egui::DragValue::new(&mut a).speed(0.01));
//             let new = [r as f32, g as f32, b as f32, a as f32];
//             if new != self.base_color {
//                 self.base_color = new;
//                 changed = true;
//             }
//         });
//
//         ui.horizontal(|ui| {
//             ui.label("Metallic:");
//             let before = self.metallic;
//             (&mut self.metallic).inspect_value(ui);
//             if (self.metallic - before).abs() > f32::EPSILON {
//                 changed = true;
//             }
//         });
//
//         ui.horizontal(|ui| {
//             ui.label("Roughness:");
//             let before = self.roughness;
//             (&mut self.roughness).inspect_value(ui);
//             if (self.roughness - before).abs() > f32::EPSILON {
//                 changed = true;
//             }
//         });
//
//         ui.horizontal(|ui| {
//             ui.label("Emissive:");
//             let mut r = self.emissive[0] as f64;
//             let mut g = self.emissive[1] as f64;
//             let mut b = self.emissive[2] as f64;
//             ui.add(egui::DragValue::new(&mut r).speed(0.01));
//             ui.add(egui::DragValue::new(&mut g).speed(0.01));
//             ui.add(egui::DragValue::new(&mut b).speed(0.01));
//             let new = [r as f32, g as f32, b as f32];
//             if new != self.emissive {
//                 self.emissive = new;
//                 changed = true;
//             }
//         });
//
//         ui.horizontal(|ui| {
//             ui.label("Alpha Mode:");
//             let mut mode = match self.alpha_mode {
//                 AlphaMode::Opaque => 0usize,
//                 AlphaMode::Mask => 1usize,
//                 AlphaMode::Blend => 2usize,
//             };
//             egui::ComboBox::from_label("")
//                 .selected_text(match mode {
//                     0 => "OPAQUE",
//                     1 => "MASK",
//                     _ => "BLEND",
//                 })
//                 .show_ui(ui, |ui| {
//                     ui.selectable_value(&mut mode, 0, "OPAQUE");
//                     ui.selectable_value(&mut mode, 1, "MASK");
//                     ui.selectable_value(&mut mode, 2, "BLEND");
//                 });
//             let new_mode = match mode {
//                 0 => AlphaMode::Opaque,
//                 1 => AlphaMode::Mask,
//                 _ => AlphaMode::Blend,
//             };
//             if new_mode != self.alpha_mode {
//                 self.alpha_mode = new_mode;
//                 changed = true;
//             }
//         });
//
//         ui.horizontal(|ui| {
//             ui.label("Alpha Cutoff:");
//             let before = self.alpha_cutoff;
//             (&mut self.alpha_cutoff).inspect_value(ui);
//             if (self.alpha_cutoff - before).abs() > f32::EPSILON {
//                 changed = true;
//             }
//         });
//
//         ui.horizontal(|ui| {
//             ui.label("Double Sided:");
//             let before = self.double_sided;
//             (&mut self.double_sided).inspect_value(ui);
//             if self.double_sided != before {
//                 changed = true;
//             }
//         });
//
//         ui.horizontal(|ui| {
//             ui.label("Albedo Texture:");
//             (&mut self.albedo_texture_name).inspect_value(ui);
//         });
//
//         ui.horizontal(|ui| {
//             ui.label("Metallic Texture:");
//             (&mut self.metallic_texture_name).inspect_value(ui);
//         });
//
//         ui.horizontal(|ui| {
//             ui.label("Normal Texture:");
//             (&mut self.normal_texture_name).inspect_value(ui);
//         });
//
//         ui.horizontal(|ui| {
//             ui.label("Emissive Texture:");
//             (&mut self.emmisive_texture_name).inspect_value(ui);
//         });
//
//         if changed {
//             self.on_inspect();
//         }
//         changed
//     }
//
//     fn on_inspect(&mut self) {}
// }
//
// impl InspectValue for Material {
//     fn inspect_value(&mut self, ui: &mut egui::Ui) {
//         let _ = Inspectable::inspect(self, ui);
//     }
// }
