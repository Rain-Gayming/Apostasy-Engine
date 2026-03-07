use std::{path::Path, sync::Arc};

use ash::vk::{self};

use crate::engine::{
    assets::{
        asset::{AssetLoadError, AssetLoader},
        handle::Handle,
        server::AssetServer,
    },
    rendering::{
        models::{
            model::{GpuMesh, GpuModel},
            vertex::{Vertex, VertexType},
        },
        rendering_context::RenderingContext,
    },
};

pub struct GltfLoader {
    pub context: Arc<RenderingContext>,
    pub command_pool: vk::CommandPool,
}

impl GltfLoader {
    pub fn new(context: Arc<RenderingContext>, command_pool: vk::CommandPool) -> Self {
        Self {
            context,
            command_pool,
        }
    }
}

impl AssetLoader for GltfLoader {
    type Asset = GpuModel;

    fn extensions(&self) -> &[&str] {
        &["glb", "gltf"]
    }

    fn load_sync(&self, path: &Path) -> Result<GpuModel, AssetLoadError> {
        let path_str = path
            .to_str()
            .ok_or_else(|| AssetLoadError::other("Non-UTF-8 path"))?;

        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("model")
            .to_string();

        let (gltf, buffers, _images) =
            gltf::import(path_str).map_err(|e| AssetLoadError::Parse {
                path: path_str.to_string(),
                message: e.to_string(),
            })?;

        let mut meshes = Vec::new();

        println!("Loading gltf: {}", path_str);

        for mesh in gltf.meshes() {
            for primitive in mesh.primitives() {
                let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

                let positions = reader
                    .read_positions()
                    .ok_or_else(|| AssetLoadError::Parse {
                        path: path_str.to_string(),
                        message: "Mesh has no vertex positions".into(),
                    })?
                    .collect::<Vec<_>>();

                let normals = reader
                    .read_normals()
                    .ok_or_else(|| AssetLoadError::Parse {
                        path: path_str.to_string(),
                        message: "Mesh has no normals".into(),
                    })?
                    .collect::<Vec<_>>();

                let tex_coords = reader
                    .read_tex_coords(0)
                    .ok_or_else(|| AssetLoadError::Parse {
                        path: path_str.to_string(),
                        message: "Mesh has no UV coordinates".into(),
                    })?
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
                    .ok_or_else(|| AssetLoadError::Parse {
                        path: path_str.to_string(),
                        message: "Mesh has no indices".into(),
                    })?
                    .into_u32()
                    .collect::<Vec<_>>();

                let vertex_buffer = self
                    .context
                    .create_vertex_buffer(vertices.as_slice(), self.command_pool)
                    .map_err(|e| AssetLoadError::other(e.to_string()))?;

                let index_buffer = self
                    .context
                    .create_index_buffer(&indices, self.command_pool)
                    .map_err(|e| AssetLoadError::other(e.to_string()))?;

                let material_name = primitive
                    .material()
                    .name()
                    .unwrap_or("material")
                    .to_string();

                meshes.push(GpuMesh {
                    vertex_buffer: vertex_buffer.0,
                    vertex_buffer_memory: vertex_buffer.1,
                    index_buffer: index_buffer.0,
                    index_buffer_memory: index_buffer.1,
                    index_count: indices.len() as u32,
                    vertex_type: VertexType::Model,
                    material_name: material_name,
                });
            }
        }

        Ok(GpuModel { name, meshes })
    }
}

pub fn preload_models_from_dir(
    server: &AssetServer,
    dir: &str,
) -> std::collections::HashMap<String, Handle<GpuModel>> {
    let mut map = std::collections::HashMap::new();

    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("[AssetServer] Failed to read model dir '{}': {}", dir, e);
            return map;
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            continue;
        }
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        if ext != "glb" && ext != "gltf" {
            continue;
        }

        let file_name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n.to_string(),
            None => continue,
        };

        match server.load_cached::<GpuModel>(&path) {
            Ok(handle) => {
                map.insert(file_name, handle);
            }
            Err(e) => {
                eprintln!(
                    "[AssetServer] Failed to load model '{}': {}",
                    path.display(),
                    e
                );
            }
        }
    }

    map
}
