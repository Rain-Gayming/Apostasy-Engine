use std::fs;

use crate::{
    self as apostasy,
    engine::{ecs::World, rendering::rendering_context::RenderingContext},
};
use anyhow::Result;
use apostasy_macros::{Component, Resource, start};
use ash::vk;
use egui::ahash::HashMap;

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
pub struct Mesh {
    pub vertex_buffer: vk::Buffer,
    pub vertex_buffer_memory: vk::DeviceMemory,
    pub index_buffer: vk::Buffer,
    pub index_buffer_memory: vk::DeviceMemory,
    pub index_count: u32,
}

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
            });
        }
    }

    Ok(Model { meshes })
}

#[repr(C)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub tex_coord: [f32; 2],
}
impl Vertex {
    pub fn get_binding_description() -> vk::VertexInputBindingDescription {
        vk::VertexInputBindingDescription::default()
            .binding(0)
            .stride(std::mem::size_of::<Vertex>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX)
    }

    pub fn get_attribute_descriptions() -> [vk::VertexInputAttributeDescription; 3] {
        [
            // Position
            vk::VertexInputAttributeDescription::default()
                .binding(0)
                .location(0)
                .format(vk::Format::R32G32B32_SFLOAT)
                .offset(0),
            // Normal
            vk::VertexInputAttributeDescription::default()
                .binding(0)
                .location(1)
                .format(vk::Format::R32G32B32_SFLOAT)
                .offset(12), // 3 floats * 4 bytes
            // Tex Coord
            vk::VertexInputAttributeDescription::default()
                .binding(0)
                .location(2)
                .format(vk::Format::R32G32_SFLOAT)
                .offset(24), // 6 floats * 4 bytes
        ]
    }
}
