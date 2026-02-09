use std::fs;

use crate::{self as apostasy, engine::rendering::rendering_context::RenderingContext};
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

fn load_material(gltf_material: &gltf::Material, textures: &[Option<Texture>]) -> Result<Material> {
    let pbr = gltf_material.pbr_metallic_roughness();

    let base_color_texture = pbr
        .base_color_texture()
        .and_then(|info| textures.get(info.texture().index()))
        .and_then(|t| t.clone());

    let metallic_roughness_texture = pbr
        .metallic_roughness_texture()
        .and_then(|info| textures.get(info.texture().index()))
        .and_then(|t| t.clone());

    let normal_texture = gltf_material
        .normal_texture()
        .and_then(|info| textures.get(info.texture().index()))
        .and_then(|t| t.clone());

    let emissive_texture = gltf_material
        .emissive_texture()
        .and_then(|info| textures.get(info.texture().index()))
        .and_then(|t| t.clone());

    Ok(Material {
        base_color: pbr.base_color_factor(),
        metallic: pbr.metallic_factor(),
        roughness: pbr.roughness_factor(),
        emissive: gltf_material.emissive_factor(),
        alpha_mode: gltf_material.alpha_mode(),
        alpha_cutoff: gltf_material.alpha_cutoff().unwrap_or(0.5),
        double_sided: gltf_material.double_sided(),
        base_color_texture,
        metallic_roughness_texture,
        normal_texture,
        emissive_texture,
    })
}

fn load_texture(image: &gltf::image::Data, context: &RenderingContext) -> Result<Texture> {
    let width = image.width;
    let height = image.height;
    let pixels = &image.pixels;

    // Convert image format to RGBA if needed
    let rgba_pixels = match image.format {
        gltf::image::Format::R8G8B8A8 => pixels.clone(),
        gltf::image::Format::R8G8B8 => {
            // Convert RGB to RGBA
            let mut rgba = Vec::with_capacity(width as usize * height as usize * 4);
            for chunk in pixels.chunks(3) {
                rgba.push(chunk[0]);
                rgba.push(chunk[1]);
                rgba.push(chunk[2]);
                rgba.push(255); // Alpha
            }
            rgba
        }
        gltf::image::Format::R8G8 => {
            // Convert RG to RGBA
            let mut rgba = Vec::with_capacity(width as usize * height as usize * 4);
            for chunk in pixels.chunks(2) {
                rgba.push(chunk[0]);
                rgba.push(chunk[1]);
                rgba.push(0);
                rgba.push(255);
            }
            rgba
        }
        gltf::image::Format::R8 => {
            // Convert R to RGBA
            let mut rgba = Vec::with_capacity(width as usize * height as usize * 4);
            for &r in pixels {
                rgba.push(r);
                rgba.push(r);
                rgba.push(r);
                rgba.push(255);
            }
            rgba
        }
        _ => {
            return Err(anyhow::anyhow!("Unsupported image format"));
        }
    };

    // Create texture using your rendering context
    // You'll need to add this method to RenderingContext
    context.create_texture(&rgba_pixels, width, height)
}

impl Default for Material {
    fn default() -> Self {
        Self {
            base_color: [1.0, 1.0, 1.0, 1.0],
            metallic: 0.0,
            roughness: 1.0,
            emissive: [0.0, 0.0, 0.0],
            alpha_mode: AlphaMode::Opaque,
            alpha_cutoff: 0.5,
            double_sided: false,
            base_color_texture: None,
            metallic_roughness_texture: None,
            normal_texture: None,
            emissive_texture: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Mesh {
    pub vertex_buffer: vk::Buffer,
    pub vertex_buffer_memory: vk::DeviceMemory,
    pub index_buffer: vk::Buffer,
    pub index_buffer_memory: vk::DeviceMemory,
    pub index_count: u32,
    pub material: Material,
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

            // Load material
            // let gltf_material = primitive.material();
            // let pbr = gltf_material.pbr_metallic_roughness();
            //
            // let material = Material {
            //     base_color: pbr.base_color_factor(),
            //     metallic: pbr.metallic_factor(),
            //     roughness: pbr.roughness_factor(),
            //     emissive: gltf_material.emissive_factor(),
            //     alpha_mode: gltf_material.alpha_mode(),
            //     alpha_cutoff: gltf_material.alpha_cutoff().unwrap_or(0.5),
            //     double_sided: gltf_material.double_sided(),
            //     texture_indices: [],
            //     base_color_texture: pbr.base_color_texture().map(|t| t.texture().index()),
            //     metallic_roughness_texture: pbr
            //         .metallic_roughness_texture()
            //         .map(|t| t.texture().index()),
            //     normal_texture: gltf_material.normal_texture().map(|t| t.texture().index()),
            //     emissive_texture: gltf_material
            //         .emissive_texture()
            //         .map(|t| t.texture().index()),
            // };

            // Create buffers
            let vertex_buffer = context.create_vertex_buffer(vertices.as_slice())?;
            let index_buffer = context.create_index_buffer(&indices)?;

            meshes.push(Mesh {
                vertex_buffer: vertex_buffer.0,
                vertex_buffer_memory: vertex_buffer.1,
                index_buffer: index_buffer.0,
                index_buffer_memory: index_buffer.1,
                index_count: indices.len() as u32,
                material,
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
