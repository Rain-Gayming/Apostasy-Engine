use crate as apostasy;
use std::fs;
use std::path::Path;

use crate::engine::editor::inspectable::Inspectable;
use crate::engine::rendering::{
    models::vertex::{Vertex, VertexType},
    rendering_context::RenderingContext,
};
use crate::log;
use anyhow::Result;
use apostasy_macros::{Component, InspectValue, Inspectable, SerializableComponent};
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

#[derive(Debug, Clone)]
pub struct Material {
    pub name: String,
    pub base_color: [f32; 4],
    pub metallic: f32,
    pub roughness: f32,
    pub emissive: [f32; 3],
    pub alpha_mode: AlphaMode,
    pub alpha_cutoff: f32,
    pub double_sided: bool,
    pub albedo_texture_name: Option<String>,
    albedo_color_texture: Option<Texture>,
    pub metallic_texture_name: Option<String>,
    metallic_texture: Option<Texture>,
    pub roughness_texture_name: Option<String>,
    roughness_texture: Option<Texture>,
    pub normal_texture_name: Option<String>,
    normal_texture: Option<Texture>,
    pub emmisive_texture_name: Option<String>,
    emissive_texture: Option<Texture>,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            name: "material".to_string(),
            base_color: [0.0, 0.0, 0.0, 1.0].into(),
            metallic: 0.0,
            roughness: 0.0,
            emissive: [0.0, 0.0, 0.0].into(),
            alpha_mode: AlphaMode::Opaque,
            alpha_cutoff: 0.5,
            double_sided: false,
            albedo_texture_name: Some("temp.png".to_string()),
            albedo_color_texture: None,
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

#[derive(
    Component, Clone, Inspectable, InspectValue, Serialize, Deserialize, SerializableComponent,
)]
pub struct ModelRenderer {
    pub loading_model: String,
    pub loaded_model: String,
}

impl Default for ModelRenderer {
    fn default() -> Self {
        Self {
            loading_model: "cube".to_string(),
            loaded_model: "cube".to_string(),
        }
    }
}

const ENGINE_MATERIAL_LOCATION: &str = "res/assets/materials/";
const ENGINE_TEXTURE_LOCATION: &str = "res/assets/textures/";
impl Material {
    pub fn albedo_texture(&mut self) -> &mut Option<Texture> {
        let path = ENGINE_TEXTURE_LOCATION.to_string() + &self.albedo_texture_name.clone().unwrap();
        if !Path::new(&path).exists() {
            panic!("Texture {} does not exist", path);
        }

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

pub fn load_models(model_loader: &mut ModelLoader, context: &RenderingContext) {
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
                    .insert(name, load_model(path, context).unwrap());
            }
        }
    }
}

/// Loads a model, path should be the file name, default path is "/res/models/"
pub fn load_model(path: &str, context: &RenderingContext) -> Result<Model> {
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

            // Create buffers
            let vertex_buffer = context.create_vertex_buffer(vertices.as_slice())?;
            let index_buffer = context.create_index_buffer(&indices)?;

            let material = Material::default();

            meshes.push(Mesh {
                vertex_buffer: vertex_buffer.0,
                vertex_buffer_memory: vertex_buffer.1,
                index_buffer: index_buffer.0,
                index_buffer_memory: index_buffer.1,
                index_count: indices.len() as u32,
                vertex_type: VertexType::Model,
                material: material.name.to_string(),
            });
        }
    }

    Ok(Model { meshes })
}
