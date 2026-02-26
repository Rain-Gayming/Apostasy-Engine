use crate as apostasy;
use std::fs;

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
}

#[derive(Clone, Debug)]
pub struct Model {
    pub meshes: Vec<Mesh>,
}

#[derive(Debug, Clone)]
pub struct Material {
    pub base_color: [f32; 4],
    pub metallic: f32,
    pub roughness: f32,
    pub emissive: [f32; 3],
    pub alpha_mode: AlphaMode,
    pub alpha_cutoff: f32,
    pub double_sided: bool,
    pub texture_name: Option<String>,
    pub base_color_texture: Option<Texture>,
    pub metallic_roughness_texture: Option<Texture>,
    pub normal_texture: Option<Texture>,
    pub emissive_texture: Option<Texture>,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            base_color: [0.0, 0.0, 0.0, 1.0],
            metallic: 0.0,
            roughness: 0.0,
            emissive: [0.0, 0.0, 0.0],
            alpha_mode: AlphaMode::Opaque,
            alpha_cutoff: 0.5,
            double_sided: false,
            texture_name: Some("temp.png".to_string()),
            base_color_texture: None,
            metallic_roughness_texture: None,
            normal_texture: None,
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
    pub material: Material,
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

pub fn does_model_exist(name: &str, model_loader: &ModelLoader) -> bool {
    model_loader.models.contains_key(name)
}

pub fn get_model<'a>(name: &'a str, model_loader: &'a mut ModelLoader) -> Option<&'a mut Model> {
    model_loader.models.get_mut(name)
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
                material,
            });
        }
    }

    Ok(Model { meshes })
}
