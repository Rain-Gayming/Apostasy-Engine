use std::fs;

use crate::{
    self as apostasy,
    engine::rendering::{models::vertex::Vertex, rendering_context::RenderingContext},
};
use anyhow::Result;
use apostasy_macros::{Component, Resource};
use ash::vk;
use egui::ahash::HashMap;
use gltf::material::AlphaMode;

const MODEL_LOCATION: &str = "res/models/";

#[derive(Resource, Default)]
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
    pub base_color_texture: Option<Texture>,
    pub metallic_roughness_texture: Option<Texture>,
    pub normal_texture: Option<Texture>,
    pub emissive_texture: Option<Texture>,
}
#[derive(Debug, Clone)]
pub struct Texture {
    pub image: vk::Image,
    pub image_memory: vk::DeviceMemory,
    pub image_view: vk::ImageView,
    pub sampler: vk::Sampler,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone)]
pub struct Mesh {
    pub vertex_buffer: vk::Buffer,
    pub vertex_buffer_memory: vk::DeviceMemory,
    pub index_buffer: vk::Buffer,
    pub index_buffer_memory: vk::DeviceMemory,
    pub index_count: u32,
    // pub material: Material,
}

#[derive(Component)]
pub struct MeshRenderer(pub Mesh);

#[derive(Component)]
pub struct ModelRenderer(pub String);

pub fn get_model(name: &str, model_loader: &ModelLoader) -> Model {
    // println!("getting model: {}", name);
    // println!("models: {:?}", model_loader.models.keys());
    model_loader.models.get(name).unwrap().clone()
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
    println!("loading model: {}", path);
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

            meshes.push(Mesh {
                vertex_buffer: vertex_buffer.0,
                vertex_buffer_memory: vertex_buffer.1,
                index_buffer: index_buffer.0,
                index_buffer_memory: index_buffer.1,
                index_count: indices.len() as u32,
                // material,
            });
        }
    }

    Ok(Model { meshes })
}
