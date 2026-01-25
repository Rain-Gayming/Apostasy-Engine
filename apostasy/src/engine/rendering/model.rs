use crate::{
    self as apostasy,
    engine::{ecs::World, rendering::rendering_context::RenderingContext},
};
use anyhow::Result;
use apostasy_macros::{Component, Resource, update};
use ash::vk;
use egui::ahash::HashMap;

const MODEL_LOCATION: &str = "res/models/";

#[derive(Resource)]
pub struct ModelLoader {
    pub models: HashMap<String, Model>,
}

#[derive(Clone, Debug)]
pub struct Model {
    pub meshes: Vec<Mesh>,
}

#[derive(Debug, Clone)]
pub struct Mesh {
    vertex_buffer: vk::Buffer,
    vertex_buffer_memory: vk::DeviceMemory,
    index_buffer: vk::Buffer,
    index_buffer_memory: vk::DeviceMemory,
    index_count: u32,
}

#[derive(Component)]
pub struct ModelRenderer(pub String);

pub fn get_model(name: &str, model_loader: &ModelLoader) -> Model {
    model_loader.models.get(name).unwrap().clone()
}

#[update]
pub fn load_models(world: &mut World) {
    world
        .query()
        .include::<ModelRenderer>()
        .build()
        .run(|entity| {
            world.with_resource_mut::<ModelLoader, _>(|model_loader| {
                let model = get_model(&entity.get::<ModelRenderer>().unwrap().0, model_loader);
                println!("{:?}", model);
            });
        });
}

/// Loads a model, path should be the file name, default path is "/res/models/"
pub fn load_model(path: &str, context: RenderingContext) -> Result<Model> {
    let path = format!("{}{}", MODEL_LOCATION, path);
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

pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub tex_coord: [f32; 2],
}
