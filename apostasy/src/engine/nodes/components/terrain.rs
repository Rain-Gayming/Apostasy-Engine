use ash::vk;
use cgmath::{InnerSpace, Vector2};

use crate::engine::editor::inspectable::Inspectable as InspectableTrait;
use crate::engine::nodes::components::camera::Camera;
use crate::engine::nodes::components::transform::Transform;
use crate::engine::nodes::world::World;
use crate::{self as apostasy, log};
use apostasy_macros::{Component, InspectValue, Inspectable, SerializableComponent, update};
use serde::{Deserialize, Serialize};

#[derive(
    Component, Clone, Inspectable, InspectValue, SerializableComponent, Serialize, Deserialize,
)]
pub struct Terrain {
    /// How many sub divisions per chunk
    pub subdivisions: u8,
    /// How far between each vertex on a chunk
    pub world_scale: f32,
    pub lod_levels: u8,

    #[inspect(skip)]
    #[serde(default)]
    pub chunks: Vec<TerrainChunk>,
    #[serde(skip, default)]
    pub is_dirty: bool,

    #[inspect(skip)]
    #[serde(skip, default)]
    pub gpu_chunks: Vec<TerrainChunkGpu>,
}

impl Default for Terrain {
    fn default() -> Self {
        Self {
            subdivisions: 16,
            world_scale: 1.0,
            lod_levels: 4,
            chunks: Vec::new(),
            is_dirty: false,

            gpu_chunks: Vec::new(),
        }
    }
}

impl Terrain {
    pub fn apply_brush(&mut self, chunk_index: usize, x: u32, z: u32, radius: u32, delta: f32) {
        if let Some(chunk) = self.chunks.get_mut(chunk_index) {
            let subdivisions = self.subdivisions.max(1) as i32;
            let center_x = x as i32;
            let center_z = z as i32;
            let radius = radius as i32;

            for dz in -radius..=radius {
                for dx in -radius..=radius {
                    let nx = center_x + dx;
                    let nz = center_z + dz;
                    if nx < 0 || nz < 0 || nx >= subdivisions || nz >= subdivisions {
                        continue;
                    }
                    let idx = (nz as usize) * (subdivisions as usize) + nx as usize;
                    if let Some(value) = chunk.heightmap.get_mut(idx) {
                        *value += delta;
                    }
                }
            }
            chunk.mesh_handle = None;
            chunk.gpu_dirty = true;
        }
    }
}

#[derive(Clone, InspectValue, Inspectable, Serialize, Deserialize)]
pub struct TerrainChunk {
    pub origin: Vector2<i32>,
    pub lod: u8,
    #[serde(skip, default)]
    #[inspect(skip)]
    pub mesh_handle: Option<TerrainMesh>,
    #[serde(default)]
    pub heightmap: Vec<f32>,
    #[serde(skip, default = "default_true")]
    #[inspect(skip)]
    pub gpu_dirty: bool,
}

fn default_true() -> bool {
    true
}

#[update]
pub fn terrain_update_system(world: &mut World) {
    let camera_pos = world
        .get_global_node_with_component::<Camera>()
        .and_then(|n| n.get_component::<Transform>())
        .map(|t| t.global_position)
        .unwrap();

    let terrain_ids: Vec<u64> = world
        .get_all_world_nodes()
        .iter()
        .filter(|n| n.has_component::<Terrain>())
        .map(|n| n.id)
        .collect();

    for id in terrain_ids {
        let node = world.get_node_mut(id);
        if let Some(_) = node.get_component::<Transform>() {
            let (terrain, transform) = node.get_components_mut::<(&mut Terrain, &mut Transform)>();

            if terrain.chunks.is_empty() {
                terrain.is_dirty = true;
            }

            if terrain.is_dirty {
                regenerate_chunks(terrain);
                log!("Regenerating terrain");
                terrain.is_dirty = false;
            }

            update_lod(terrain, transform, camera_pos);

            let subdivisions = terrain.subdivisions;
            let world_scale = terrain.world_scale;
            let required_heightmap_len = (subdivisions.max(1) as usize).pow(2);

            for chunk in terrain.chunks.iter_mut() {
                if chunk.heightmap.len() != required_heightmap_len {
                    chunk.heightmap.resize(required_heightmap_len, 0.0);
                    chunk.mesh_handle = None;
                    chunk.gpu_dirty = true;
                }
                if chunk.mesh_handle.is_none() {
                    chunk.mesh_handle = Some(build_terrain_mesh(subdivisions, world_scale, chunk));
                }
            }
        }
    }
}

fn regenerate_chunks(terrain: &mut Terrain) {
    terrain.chunks.clear();
    let count = 4i32;
    let subdivisions = terrain.subdivisions.max(1) as usize;
    let heightmap = vec![0.0; subdivisions * subdivisions];
    for cz in 0..count {
        for cx in 0..count {
            terrain.chunks.push(TerrainChunk {
                origin: Vector2::new(cx, cz),
                lod: 0,
                mesh_handle: None,
                heightmap: heightmap.clone(),
                gpu_dirty: true,
            });
        }
    }
}

fn update_lod(terrain: &mut Terrain, transform: &Transform, camera_pos: cgmath::Vector3<f32>) {
    let chunk_world_size = terrain.subdivisions as f32 * terrain.world_scale;

    for chunk in terrain.chunks.iter_mut() {
        let chunk_world_x = chunk.origin.x as f32 * chunk_world_size + transform.global_position.x;
        let chunk_world_z = chunk.origin.y as f32 * chunk_world_size + transform.global_position.z;

        let dist = cgmath::Vector2::new(camera_pos.x - chunk_world_x, camera_pos.z - chunk_world_z)
            .magnitude();

        let new_lod = match dist {
            d if d < 64.0 => 0,
            d if d < 128.0 => 1,
            d if d < 256.0 => 2,
            _ => 3,
        };

        if new_lod != chunk.lod {
            chunk.lod = new_lod;
            chunk.mesh_handle = None;
        }
    }
}

#[derive(Clone, Default, InspectValue, Inspectable, Serialize, Deserialize)]
pub struct TerrainMesh {
    pub vertices: Vec<TerrainVertex>,
    pub indices: Vec<u32>,
}

#[repr(C)]
#[derive(Clone, Default, InspectValue, Inspectable, Serialize, Deserialize)]
pub struct TerrainVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
}

#[derive(Clone)]
pub struct TerrainChunkGpu {
    pub vertex_buffer: vk::Buffer,
    pub vertex_buffer_memory: vk::DeviceMemory,
    pub index_buffer: vk::Buffer,
    pub index_buffer_memory: vk::DeviceMemory,
    pub index_count: u32,
}

impl Default for TerrainChunkGpu {
    fn default() -> Self {
        Self {
            vertex_buffer: vk::Buffer::null(),
            vertex_buffer_memory: vk::DeviceMemory::null(),
            index_buffer: vk::Buffer::null(),
            index_buffer_memory: vk::DeviceMemory::null(),
            index_count: 0,
        }
    }
}

fn sample_heightmap(height_map: &[f32], size: u32, x: u32, z: u32) -> f32 {
    let index = (z * size + x) as usize;
    height_map.get(index).copied().unwrap_or(0.0)
}

/// How many heightmap cells to skip per vertex in this level of detail
fn lod_step(lod: u8) -> u32 {
    1u32 << lod
}

fn calculate_normal(height_map: &[f32], size: u32, x: u32, z: u32, world_scale: f32) -> [f32; 3] {
    let height_left = sample_heightmap(height_map, size, x.saturating_sub(1), z);
    let height_right = sample_heightmap(height_map, size, (x + 1).min(size - 1), z);
    let height_up = sample_heightmap(height_map, size, x, z.saturating_sub(1));
    let height_down = sample_heightmap(height_map, size, x, (z + 1).min(size - 1));

    let dx = (height_right - height_left) / (2.0 * world_scale);
    let dz = (height_up - height_down) / (2.0 * world_scale);

    let nx = -dx;
    let ny = 1.0_f32;
    let nz = -dz;
    let len = (nx * nx + ny * ny + nz * nz).sqrt();
    [nx / len, ny / len, nz / len]
}

fn build_terrain_mesh(subdivisions: u8, world_scale: f32, chunk: &TerrainChunk) -> TerrainMesh {
    let step = lod_step(chunk.lod);
    let vertices_per_side = (subdivisions as u32 / step) + 1;
    let vertex_spacing = world_scale * step as f32; // world units between vertices at this lod

    let world_offset_x = chunk.origin.x as f32 * subdivisions as f32 * world_scale;
    let world_offset_z = chunk.origin.y as f32 * subdivisions as f32 * world_scale;

    let mut vertices: Vec<TerrainVertex> =
        Vec::with_capacity((vertices_per_side * vertices_per_side) as usize);
    let mut indices: Vec<u32> = Vec::with_capacity(((vertices_per_side - 1).pow(2) * 6) as usize);

    for row in 0..vertices_per_side {
        for col in 0..vertices_per_side {
            let height_x = (col * step).min(subdivisions as u32 - 1);
            let height_z = (row * step).min(subdivisions as u32 - 1);

            // fix: use height_z not height_x for the z parameter
            let height =
                sample_heightmap(&chunk.heightmap, subdivisions as u32, height_x, height_z);

            let x = world_offset_x + col as f32 * vertex_spacing;
            let z = world_offset_z + row as f32 * vertex_spacing;

            let normal = calculate_normal(
                &chunk.heightmap,
                subdivisions as u32,
                height_x,
                height_z,
                world_scale,
            );

            let u = col as f32 / (vertices_per_side - 1) as f32;
            let v = row as f32 / (vertices_per_side - 1) as f32;

            vertices.push(TerrainVertex {
                position: [x, height, z],
                normal,
                uv: [u, v],
            });
        }
    }

    for row in 0..(vertices_per_side - 1) {
        for col in 0..(vertices_per_side - 1) {
            let tl = row * vertices_per_side + col;
            let tr = tl + 1;
            let bl = tl + vertices_per_side;
            let br = bl + 1;
            indices.extend_from_slice(&[tl, bl, tr, tr, bl, br]);
        }
    }

    TerrainMesh { vertices, indices }
}
