use anyhow::Result;
use apostasy_macros::{Component, Tag};
use ash::vk::Buffer;
use ash::vk::{self, CommandPool, DeviceMemory};
use hashbrown::HashMap;

use crate::log;
use crate::objects::scene::ObjectId;
use crate::objects::world::World;
use crate::rendering::shared::model::GpuMesh;
use crate::rendering::shared::vertex::VertexDefinition;
use crate::rendering::vulkan::rendering_context::VulkanRenderingContext;
use crate::utils::flatten::flatten;
use crate::voxels::VoxelTransform;
use crate::voxels::chunk::Chunk;
use crate::voxels::voxel::VoxelRegistry;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct VoxelVertex {
    pub data_lo: u32,
    pub data_hi: u32,
}

impl VoxelVertex {
    pub fn pack(
        x: u8,
        y: u8,
        z: u8,
        face: u8,
        u: u8,
        v: u8,
        texture_id: u16,
        quad_w: u8,
        quad_h: u8,
    ) -> Self {
        let data_lo: u32 = (x as u32)
            | ((y as u32) << 6)
            | ((z as u32) << 12)
            | ((face as u32) << 18)
            | ((u as u32) << 21)
            | ((v as u32) << 27);
        let data_hi: u32 = (texture_id as u32) | ((quad_w as u32) << 16) | ((quad_h as u32) << 24);
        Self { data_lo, data_hi }
    }
}

impl VertexDefinition for VoxelVertex {
    fn get_binding_description() -> vk::VertexInputBindingDescription {
        vk::VertexInputBindingDescription::default()
            .binding(0)
            .stride(std::mem::size_of::<VoxelVertex>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX)
    }

    fn get_attribute_descriptions() -> Vec<vk::VertexInputAttributeDescription> {
        vec![
            vk::VertexInputAttributeDescription::default()
                .binding(0)
                .location(0)
                .format(vk::Format::R32_UINT)
                .offset(0),
            vk::VertexInputAttributeDescription::default()
                .binding(0)
                .location(1)
                .format(vk::Format::R32_UINT)
                .offset(4),
        ]
    }
}

#[derive(Debug, Component, Clone, Default)]
pub struct VoxelChunkMesh {
    pub vertex_buffer: Buffer,
    pub vertex_buffer_memory: DeviceMemory,
    pub index_buffer: Buffer,
    pub index_buffer_memory: DeviceMemory,
    pub index_count: u32,
}

impl VoxelChunkMesh {
    pub fn deserialize(&mut self, _value: &serde_yaml::Value) -> anyhow::Result<()> {
        Ok(())
    }
}

#[derive(Debug, Tag, Clone, Default)]
pub struct NeedsRemeshing;

impl GpuMesh for VoxelChunkMesh {
    fn get_vertex_buffer(&self) -> Buffer {
        self.vertex_buffer
    }
    fn get_index_buffer(&self) -> Buffer {
        self.index_buffer
    }
    fn get_index_count(&self) -> u32 {
        self.index_count
    }
}

pub struct ChunkNeighbours<'a> {
    pub px: Option<&'a Chunk>, // +X
    pub nx: Option<&'a Chunk>, // -X
    pub py: Option<&'a Chunk>, // +Y
    pub ny: Option<&'a Chunk>, // -Y
    pub pz: Option<&'a Chunk>, // +Z
    pub nz: Option<&'a Chunk>, // -Z
}

impl<'a> ChunkNeighbours<'a> {
    pub fn empty() -> Self {
        Self {
            px: None,
            nx: None,
            py: None,
            ny: None,
            pz: None,
            nz: None,
        }
    }
}
fn sample_neighbour(
    neighbours: &ChunkNeighbours,
    face: usize,
    u: usize,
    v: usize,
    lod: usize,
) -> u16 {
    let neighbour = match face {
        0 => neighbours.px,
        1 => neighbours.nx,
        2 => neighbours.py,
        3 => neighbours.ny,
        4 => neighbours.pz,
        5 => neighbours.nz,
        _ => None,
    };

    let Some(neighbour) = neighbour else {
        return 0;
    };

    let (x, y, z) = match face {
        0 => (0usize, u, v), // entering +X neighbour at x=0
        1 => (31, u, v),     // entering -X neighbour at x=31
        2 => (u, 0, v),      // entering +Y neighbour at y=0
        3 => (u, 31, v),     // entering -Y neighbour at y=31
        4 => (u, v, 0),      // entering +Z neighbour at z=0
        5 => (u, v, 31),     // entering -Z neighbour at z=31
        _ => (0, 0, 0),
    };

    get_representative_voxel(neighbour, x, y, z, lod)
}

pub fn remesh_chunks(
    world: &mut World,
    ctx: &VulkanRenderingContext,
    command_pool: CommandPool,
) -> Result<()> {
    let registry = world
        .get_resource::<VoxelRegistry>()
        .expect("No VoxelRegistry resource")
        .clone();

    let chunk_map: HashMap<(i32, i32, i32), Chunk> = world
        .get_objects_with_component::<Chunk>()
        .iter()
        .filter_map(|obj| {
            let pos = obj.get_component::<VoxelTransform>().ok()?.position;
            let chunk = obj.get_component::<Chunk>().ok()?.clone();
            Some(((pos.x, pos.y, pos.z), chunk))
        })
        .collect();
    // collect ids that need remeshing
    // collect ids that need remeshing
    let needs_remesh: Vec<ObjectId> = world
        .get_objects_with_tag_with_ids::<NeedsRemeshing>()
        .into_iter()
        .map(|(id, _o)| id)
        .collect();

    for id in needs_remesh {
        let Some(object) = world.get_object_mut(id) else {
            continue;
        };

        if !object.has_tag::<NeedsRemeshing>() {
            continue;
        }

        let Ok(chunk) = object.get_component::<Chunk>() else {
            log!("no chunk");
            continue;
        };
        let chunk = chunk.clone();

        let Ok(transform) = object.get_component::<VoxelTransform>() else {
            log!("no transform");
            continue;
        };
        let pos = transform.position;

        // look up neighbours
        let neighbours = ChunkNeighbours {
            px: chunk_map.get(&(pos.x + 1, pos.y, pos.z)),
            nx: chunk_map.get(&(pos.x - 1, pos.y, pos.z)),
            py: chunk_map.get(&(pos.x, pos.y + 1, pos.z)),
            ny: chunk_map.get(&(pos.x, pos.y - 1, pos.z)),
            pz: chunk_map.get(&(pos.x, pos.y, pos.z + 1)),
            nz: chunk_map.get(&(pos.x, pos.y, pos.z - 1)),
        };

        let (vertices, indices) = generate_mesh(&chunk, &registry, &neighbours);

        if vertices.is_empty() || indices.is_empty() {
            object.remove_tag::<NeedsRemeshing>();
            continue;
        }

        if let Ok(mesh) = object.get_component::<VoxelChunkMesh>() {
            if mesh.vertex_buffer != vk::Buffer::null() {
                unsafe {
                    ctx.device.destroy_buffer(mesh.vertex_buffer, None);
                    ctx.device.free_memory(mesh.vertex_buffer_memory, None);
                    ctx.device.destroy_buffer(mesh.index_buffer, None);
                    ctx.device.free_memory(mesh.index_buffer_memory, None);
                }
            }
        }

        let (vertex_buffer, vertex_buffer_memory) =
            ctx.create_vertex_buffer(&vertices, command_pool)?;
        let (index_buffer, index_buffer_memory) =
            ctx.create_index_buffer(&indices, command_pool)?;

        if !object.has_component::<VoxelChunkMesh>() {
            object.add_component(VoxelChunkMesh::default());
        }

        let mesh = object.get_component_mut::<VoxelChunkMesh>().unwrap();
        mesh.vertex_buffer = vertex_buffer;
        mesh.vertex_buffer_memory = vertex_buffer_memory;
        mesh.index_buffer = index_buffer;
        mesh.index_buffer_memory = index_buffer_memory;
        mesh.index_count = indices.len() as u32;

        object.remove_tag::<NeedsRemeshing>();
    }

    Ok(())
}

pub fn generate_mesh(
    chunk: &Chunk,
    registry: &VoxelRegistry,
    neighbours: &ChunkNeighbours,
) -> (Vec<VoxelVertex>, Vec<u32>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let lod = chunk.lod as usize;

    const FACE_NORMALS: [[i32; 3]; 6] = [
        [1, 0, 0],
        [-1, 0, 0],
        [0, 1, 0],
        [0, -1, 0],
        [0, 0, 1],
        [0, 0, -1],
    ];

    for z in (0..32).step_by(lod) {
        for y in (0..32).step_by(lod) {
            for x in (0..32).step_by(lod) {
                let id = get_representative_voxel(chunk, x, y, z, lod);
                if id == 0 {
                    continue;
                }

                for face in 0..6usize {
                    let normal = FACE_NORMALS[face];
                    let nx = x as i32 + normal[0] * lod as i32;
                    let ny = y as i32 + normal[1] * lod as i32;
                    let nz = z as i32 + normal[2] * lod as i32;

                    let neighbour_solid = if nx >= 0
                        && nx < 32
                        && ny >= 0
                        && ny < 32
                        && nz >= 0
                        && nz < 32
                    {
                        get_representative_voxel(chunk, nx as usize, ny as usize, nz as usize, lod)
                            != 0
                    } else {
                        let (u, v) = match face {
                            0 | 1 => (y, z), // X face, u=y v=z
                            2 | 3 => (x, z), // Y face, u=x v=z
                            4 | 5 => (x, y), // Z face, u=x v=y
                            _ => (0, 0),
                        };
                        sample_neighbour(neighbours, face, u, v, lod) != 0
                    };

                    if neighbour_solid {
                        continue;
                    }

                    let texture_id = registry.defs[id as usize]
                        .textures
                        .get_for_face(face as u8, x as u32, y as u32, z as u32);

                    let base = vertices.len() as u32;
                    let corners = face_corners(x, y, z, face, lod);

                    for (i, &(u, v)) in [(0u8, 0u8), (1, 0), (1, 1), (0, 1)].iter().enumerate() {
                        vertices.push(VoxelVertex::pack(
                            corners[i][0],
                            corners[i][1],
                            corners[i][2],
                            face as u8,
                            u,
                            v,
                            texture_id as u16,
                            1,
                            1,
                        ));
                    }

                    indices.extend_from_slice(&[
                        base,
                        base + 1,
                        base + 3,
                        base + 1,
                        base + 2,
                        base + 3,
                    ]);
                }
            }
        }
    }

    (vertices, indices)
}

fn get_representative_voxel(chunk: &Chunk, x: usize, y: usize, z: usize, lod: usize) -> u16 {
    for dz in 0..lod {
        for dy in 0..lod {
            for dx in 0..lod {
                let sx = x + dx;
                let sy = y + dy;
                let sz = z + dz;

                if sx >= 32 || sy >= 32 || sz >= 32 {
                    continue;
                }
                let id = chunk.voxels[flatten(sx as u32, sy as u32, sz as u32, 32)];
                if id != 0 {
                    return id;
                }
            }
        }
    }
    0
}
fn face_corners(x: usize, y: usize, z: usize, face: usize, lod: usize) -> [[u8; 3]; 4] {
    let x = x as u8;
    let y = y as u8;
    let z = z as u8;
    let l = lod as u8;

    match face {
        0 => [
            // +X
            [x + l, y, z],
            [x + l, y + l, z],
            [x + l, y + l, z + l],
            [x + l, y, z + l],
        ],
        1 => [
            // -X
            [x, y, z + l],
            [x, y + l, z + l],
            [x, y + l, z],
            [x, y, z],
        ],
        2 => [
            // +Y
            [x, y + l, z + l],
            [x + l, y + l, z + l],
            [x + l, y + l, z],
            [x, y + l, z],
        ],
        3 => [
            // -Y
            [x, y, z],
            [x + l, y, z],
            [x + l, y, z + l],
            [x, y, z + l],
        ],
        4 => [
            // +Z
            [x + l, y, z + l],
            [x + l, y + l, z + l],
            [x, y + l, z + l],
            [x, y, z + l],
        ],
        _ => [
            // -Z
            [x, y, z],
            [x, y + l, z],
            [x + l, y + l, z],
            [x + l, y, z],
        ],
    }
}
