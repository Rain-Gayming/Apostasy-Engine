use crate as apostasy;
use std::fs;
use std::path::Path;

use crate::engine::editor::inspectable::{InspectValue, Inspectable};
use crate::engine::rendering::{
    models::vertex::{Vertex, VertexType},
    rendering_context::RenderingContext,
};
use crate::log;
use anyhow::Result;
use apostasy_macros::{Component, SerializableComponent};
use ash::vk;
use egui::ahash::HashMap;
use gltf::material::AlphaMode;
use serde::{Deserialize, Serialize};

const MODEL_LOCATION: &str = "res/models/";

#[derive(Default, Clone)]
pub struct ModelLoader {
    pub models: HashMap<String, Model>,
    pub materials: HashMap<String, Material>,
}

#[derive(Clone, Debug)]
pub struct Model {
    pub meshes: Vec<Mesh>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Material {
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
    #[serde(skip)]
    albedo_color_texture: Option<Texture>,
    pub albedo_texture_loaded_name: Option<String>,
    pub metallic_texture_name: Option<String>,
    #[serde(skip)]
    metallic_texture: Option<Texture>,
    pub roughness_texture_name: Option<String>,
    #[serde(skip)]
    roughness_texture: Option<Texture>,
    pub normal_texture_name: Option<String>,
    #[serde(skip)]
    normal_texture: Option<Texture>,
    pub emmisive_texture_name: Option<String>,
    #[serde(skip)]
    emissive_texture: Option<Texture>,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            name: "material".to_string(),
            base_color: [1.0, 1.0, 1.0, 1.0],
            metallic: 0.0,
            roughness: 0.0,
            emissive: [0.0, 0.0, 0.0],
            alpha_mode: AlphaMode::Opaque,
            alpha_cutoff: 0.5,
            double_sided: false,
            albedo_texture_name: Some("temp.png".to_string()),
            albedo_color_texture: None,
            albedo_texture_loaded_name: None,
            metallic_texture_name: None,
            metallic_texture: None,
            roughness_texture_name: None,
            roughness_texture: None,
            normal_texture_name: None,
            normal_texture: None,
            emmisive_texture_name: None,
            emissive_texture: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Texture {
    pub image: vk::Image,
    pub image_view: vk::ImageView,
    pub memory: vk::DeviceMemory,
    pub sampler: vk::Sampler,
    pub descriptor_set: vk::DescriptorSet,
}

#[derive(Debug, Clone, Default)]
pub struct Mesh {
    pub vertex_buffer: vk::Buffer,
    pub vertex_buffer_memory: vk::DeviceMemory,
    pub index_buffer: vk::Buffer,
    pub index_buffer_memory: vk::DeviceMemory,
    pub index_count: u32,
    pub vertex_type: VertexType,
    pub material: String,
}

#[derive(Component, Clone, Serialize, Deserialize, SerializableComponent)]
pub struct ModelRenderer {
    pub loading_model: String,
    pub loaded_model: String,
    pub material: Option<Material>,
}

impl Default for ModelRenderer {
    fn default() -> Self {
        Self {
            loading_model: "cube".to_string(),
            loaded_model: "cube".to_string(),
            material: None,
        }
    }
}

impl Inspectable for ModelRenderer {
    fn inspect(&mut self, ui: &mut egui::Ui) -> bool {
        let mut remove = false;
        ui.horizontal(|ui| {
            if ui.small_button("✕").clicked() {
                remove = true;
            }
            egui::CollapsingHeader::new("ModelRenderer")
                .default_open(true)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("loading_model:");
                        (&mut self.loading_model).inspect_value(ui);
                    });
                    ui.horizontal(|ui| {
                        ui.label("loaded_model:");
                        (&mut self.loaded_model).inspect_value(ui);
                    });

                    ui.horizontal(|ui| {
                        ui.label("material:");
                        match &mut self.material {
                            Some(mat) => {
                                if ui.small_button("Remove").clicked() {
                                    self.material = None;
                                } else {
                                    egui::CollapsingHeader::new("Material")
                                        .default_open(false)
                                        .show(ui, |ui| {
                                            let _ = mat.inspect(ui);
                                        });
                                }
                            }
                            None => {
                                if ui.small_button("Add Material").clicked() {
                                    self.material = Some(Material::default());
                                }
                            }
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
    fn inspect_value(&mut self, ui: &mut egui::Ui) {
        let _ = self.inspect(ui);
    }
}

const ENGINE_MATERIAL_LOCATION: &str = "res/assets/materials/";
const ENGINE_TEXTURE_LOCATION: &str = "res/assets/textures/";

impl Material {
    pub fn albedo_texture(&mut self) -> &mut Option<Texture> {
        &mut self.albedo_color_texture
    }

    pub fn metallic_texture(&mut self) -> &mut Option<Texture> {
        &mut self.metallic_texture
    }

    pub fn roughness_texture(&mut self) -> &mut Option<Texture> {
        &mut self.roughness_texture
    }

    pub fn normal_texture(&mut self) -> &mut Option<Texture> {
        &mut self.normal_texture
    }

    pub fn emissive_texture(&mut self) -> &mut Option<Texture> {
        &mut self.emissive_texture
    }

    pub fn set_albedo_texture(&mut self, texture: Texture) {
        self.albedo_color_texture = Some(texture);
    }

    pub fn load(name: &str) -> Option<Material> {
        let path = ENGINE_MATERIAL_LOCATION.to_string() + name + ".yaml";
        if !Path::new(&path).exists() {
            return None;
        }

        let contents = std::fs::read_to_string(path).ok()?;
        #[derive(serde::Deserialize)]
        struct MaterialDef {
            name: Option<String>,
            base_color: Option<[f32; 4]>,
            metallic: Option<f32>,
            roughness: Option<f32>,
            emissive: Option<[f32; 3]>,
            alpha_mode: Option<String>,
            alpha_cutoff: Option<f32>,
            double_sided: Option<bool>,
            albedo_texture: Option<Option<String>>,
            metallic_roughness_texture: Option<Option<String>>,
            normal_texture: Option<Option<String>>,
            emissive_texture: Option<Option<String>>,
        }

        let def: MaterialDef = serde_yaml::from_str(&contents).ok()?;

        let alpha_mode = match def.alpha_mode.as_deref() {
            Some("MASK") => AlphaMode::Mask,
            Some("BLEND") => AlphaMode::Blend,
            _ => AlphaMode::Opaque,
        };

        Some(Material {
            name: def.name.unwrap_or_else(|| name.to_string()),
            base_color: def.base_color.unwrap_or([1.0, 1.0, 1.0, 1.0]),
            metallic: def.metallic.unwrap_or(0.0),
            roughness: def.roughness.unwrap_or(1.0),
            emissive: def.emissive.unwrap_or([0.0, 0.0, 0.0]),
            alpha_mode,
            alpha_cutoff: def.alpha_cutoff.unwrap_or(0.5),
            double_sided: def.double_sided.unwrap_or(false),
            albedo_texture_name: def.albedo_texture.and_then(|v| v),
            albedo_color_texture: None,
            albedo_texture_loaded_name: None,
            metallic_texture_name: def.metallic_roughness_texture.and_then(|v| v),
            metallic_texture: None,
            roughness_texture_name: None,
            roughness_texture: None,
            normal_texture_name: def.normal_texture.and_then(|v| v),
            normal_texture: None,
            emmisive_texture_name: def.emissive_texture.and_then(|v| v),
            emissive_texture: None,
        })
    }

    pub fn take_albedo_texture(&mut self) -> Option<Texture> {
        self.albedo_color_texture.take()
    }

    pub fn serialize_material(&self) {
        let mut output = serde_yaml::Mapping::new();
        output.insert(
            serde_yaml::Value::String("name".into()),
            serde_yaml::to_value(self.name.clone()).unwrap(),
        );
        output.insert(
            serde_yaml::Value::String("base_color".into()),
            serde_yaml::to_value(self.base_color).unwrap(),
        );
        output.insert(
            serde_yaml::Value::String("metallic".into()),
            serde_yaml::to_value(self.metallic).unwrap(),
        );
        output.insert(
            serde_yaml::Value::String("roughness".into()),
            serde_yaml::to_value(self.roughness).unwrap(),
        );
        output.insert(
            serde_yaml::Value::String("emissive".into()),
            serde_yaml::to_value(self.emissive).unwrap(),
        );
        output.insert(
            serde_yaml::Value::String("alpha_mode".into()),
            serde_yaml::to_value(self.alpha_mode).unwrap(),
        );
        output.insert(
            serde_yaml::Value::String("alpha_cutoff".into()),
            serde_yaml::to_value(self.alpha_cutoff).unwrap(),
        );
        output.insert(
            serde_yaml::Value::String("double_sided".into()),
            serde_yaml::to_value(self.double_sided).unwrap(),
        );
        output.insert(
            serde_yaml::Value::String("albedo_texture".into()),
            serde_yaml::to_value(self.albedo_texture_name.clone()).unwrap(),
        );
        output.insert(
            serde_yaml::Value::String("metallic_roughness_texture".into()),
            serde_yaml::to_value(self.metallic_texture_name.clone()).unwrap(),
        );
        output.insert(
            serde_yaml::Value::String("normal_texture".into()),
            serde_yaml::to_value(self.normal_texture_name.clone()).unwrap(),
        );
        output.insert(
            serde_yaml::Value::String("emissive_texture".into()),
            serde_yaml::to_value(self.emmisive_texture_name.clone()).unwrap(),
        );

        let path = ENGINE_MATERIAL_LOCATION.to_string() + &self.name + ".yaml";
        if !Path::new(&path).exists() {
            std::fs::create_dir_all(ENGINE_MATERIAL_LOCATION).unwrap();
        }
        std::fs::write(path, serde_yaml::to_string(&output).unwrap()).unwrap();
    }
}

impl Inspectable for Material {
    fn inspect(&mut self, ui: &mut egui::Ui) -> bool {
        let mut changed = false;

        ui.horizontal(|ui| {
            ui.label("Name:");
            if ui.text_edit_singleline(&mut self.name).changed() {
                changed = true;
            }
        });

        ui.horizontal(|ui| {
            ui.label("Base Color:");
            let mut r = self.base_color[0] as f64;
            let mut g = self.base_color[1] as f64;
            let mut b = self.base_color[2] as f64;
            let mut a = self.base_color[3] as f64;
            ui.add(egui::DragValue::new(&mut r).speed(0.01));
            ui.add(egui::DragValue::new(&mut g).speed(0.01));
            ui.add(egui::DragValue::new(&mut b).speed(0.01));
            ui.add(egui::DragValue::new(&mut a).speed(0.01));
            let new = [r as f32, g as f32, b as f32, a as f32];
            if new != self.base_color {
                self.base_color = new;
                changed = true;
            }
        });

        ui.horizontal(|ui| {
            ui.label("Metallic:");
            let before = self.metallic;
            (&mut self.metallic).inspect_value(ui);
            if (self.metallic - before).abs() > f32::EPSILON {
                changed = true;
            }
        });

        ui.horizontal(|ui| {
            ui.label("Roughness:");
            let before = self.roughness;
            (&mut self.roughness).inspect_value(ui);
            if (self.roughness - before).abs() > f32::EPSILON {
                changed = true;
            }
        });

        ui.horizontal(|ui| {
            ui.label("Emissive:");
            let mut r = self.emissive[0] as f64;
            let mut g = self.emissive[1] as f64;
            let mut b = self.emissive[2] as f64;
            ui.add(egui::DragValue::new(&mut r).speed(0.01));
            ui.add(egui::DragValue::new(&mut g).speed(0.01));
            ui.add(egui::DragValue::new(&mut b).speed(0.01));
            let new = [r as f32, g as f32, b as f32];
            if new != self.emissive {
                self.emissive = new;
                changed = true;
            }
        });

        ui.horizontal(|ui| {
            ui.label("Alpha Mode:");
            let mut mode = match self.alpha_mode {
                AlphaMode::Opaque => 0usize,
                AlphaMode::Mask => 1usize,
                AlphaMode::Blend => 2usize,
            };
            egui::ComboBox::from_label("")
                .selected_text(match mode {
                    0 => "OPAQUE",
                    1 => "MASK",
                    _ => "BLEND",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut mode, 0, "OPAQUE");
                    ui.selectable_value(&mut mode, 1, "MASK");
                    ui.selectable_value(&mut mode, 2, "BLEND");
                });
            let new_mode = match mode {
                0 => AlphaMode::Opaque,
                1 => AlphaMode::Mask,
                _ => AlphaMode::Blend,
            };
            if new_mode != self.alpha_mode {
                self.alpha_mode = new_mode;
                changed = true;
            }
        });

        ui.horizontal(|ui| {
            ui.label("Alpha Cutoff:");
            let before = self.alpha_cutoff;
            (&mut self.alpha_cutoff).inspect_value(ui);
            if (self.alpha_cutoff - before).abs() > f32::EPSILON {
                changed = true;
            }
        });

        ui.horizontal(|ui| {
            ui.label("Double Sided:");
            let before = self.double_sided;
            (&mut self.double_sided).inspect_value(ui);
            if self.double_sided != before {
                changed = true;
            }
        });

        ui.horizontal(|ui| {
            ui.label("Albedo Texture:");
            (&mut self.albedo_texture_name).inspect_value(ui);
        });

        ui.horizontal(|ui| {
            ui.label("Metallic Texture:");
            (&mut self.metallic_texture_name).inspect_value(ui);
        });

        ui.horizontal(|ui| {
            ui.label("Normal Texture:");
            (&mut self.normal_texture_name).inspect_value(ui);
        });

        ui.horizontal(|ui| {
            ui.label("Emissive Texture:");
            (&mut self.emmisive_texture_name).inspect_value(ui);
        });

        if changed {
            self.on_inspect();
        }
        changed
    }

    fn on_inspect(&mut self) {}
}

impl InspectValue for Material {
    fn inspect_value(&mut self, ui: &mut egui::Ui) {
        let _ = Inspectable::inspect(self, ui);
    }
}

pub fn does_model_exist(name: &str, model_loader: &ModelLoader) -> bool {
    model_loader.models.contains_key(name)
}

impl ModelLoader {
    pub fn get_material(&self, name: &str) -> Option<&Material> {
        self.materials.get(name)
    }
    pub fn get_material_mut(&mut self, name: &str) -> Option<&mut Material> {
        self.materials.get_mut(name)
    }

    pub fn get_model(&self, name: &str) -> Option<&Model> {
        self.models.get(name)
    }
}

pub fn load_models(model_loader: &mut ModelLoader, context: &RenderingContext, command_pool: vk::CommandPool) {
    for entry in fs::read_dir(MODEL_LOCATION).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        // TODO: impliment recursive loading
        if path.is_dir() {
            continue;
        }
        if let Some(extension) = path.extension() {
            let ext = extension.to_str().unwrap_or("");
            if ext == "gltf" || ext == "glb" {
                let name = path.file_name().unwrap().to_str().unwrap().to_string();
                let path = path.to_str().unwrap();
                model_loader
                    .models
                    .insert(name, load_model(path, context, command_pool).unwrap());
            }
        }
    }
}

/// Loads a model, path should be the file name, default path is "/res/models/"
pub fn load_model(path: &str, context: &RenderingContext, command_pool: vk::CommandPool) -> Result<Model> {
    log!("loading model: {}", path);
    let (gltf, buffers, _images) = gltf::import(path)?;

    let mut meshes = Vec::new();
    for mesh in gltf.meshes() {
        for primitive in mesh.primitives() {
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
            let positions = reader.read_positions().unwrap().collect::<Vec<_>>();
            let normals = reader.read_normals().unwrap().collect::<Vec<_>>();

            let tex_coords = reader
                .read_tex_coords(0)
                .unwrap()
                .into_f32()
                .collect::<Vec<_>>();

            let vertices: Vec<Vertex> = positions
                .iter()
                .zip(normals.iter())
                .zip(tex_coords.iter())
                .map(|((pos, norm), tex)| Vertex {
                    position: *pos,
                    normal: *norm,
                    tex_coord: *tex,
                })
                .collect();

            let indices = reader
                .read_indices()
                .unwrap()
                .into_u32()
                .collect::<Vec<_>>();

            // Create buffers using staging uploads to device-local memory
            let vertex_buffer = context.create_vertex_buffer(vertices.as_slice(), command_pool)?;
            let index_buffer = context.create_index_buffer(&indices, command_pool)?;

            // Determine material name from glTF primitive or fall back to 'material'
            let material_name = primitive
                .material()
                .name()
                .unwrap_or("material")
                .to_string();

            meshes.push(Mesh {
                vertex_buffer: vertex_buffer.0,
                vertex_buffer_memory: vertex_buffer.1,
                index_buffer: index_buffer.0,
                index_buffer_memory: index_buffer.1,
                index_count: indices.len() as u32,
                vertex_type: VertexType::Model,
                material: material_name,
            });
        }
    }

    Ok(Model { meshes })
}

mod alpha_mode_serde {
    use super::*;
    use serde::{Deserializer, Serializer};

    pub fn serialize<S>(mode: &AlphaMode, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = match mode {
            AlphaMode::Opaque => "OPAQUE",
            AlphaMode::Mask => "MASK",
            AlphaMode::Blend => "BLEND",
        };
        serializer.serialize_str(s)
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
