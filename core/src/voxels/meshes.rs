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
    let lod = chunk.lod as usize;
    let grid_size = 32 / lod;

    // compute voxels into easily accessable grid
    let mut grid = [0u16; 32 * 32 * 32];
    for gz in 0..grid_size {
        for gy in 0..grid_size {
            for gx in 0..grid_size {
                grid[gz * grid_size * grid_size + gy * grid_size + gx] =
                    get_representative_voxel(chunk, gx * lod, gy * lod, gz * lod, lod);
            }
        }
    }

    // get neighbours voxels on their neighbouring plain
    let mut border_px = [0u16; 32 * 32]; // [y * 32 + z]
    let mut border_nx = [0u16; 32 * 32];
    let mut border_py = [0u16; 32 * 32];
    let mut border_ny = [0u16; 32 * 32];
    let mut border_pz = [0u16; 32 * 32];
    let mut border_nz = [0u16; 32 * 32];

    // calculate the voxels on the neighbours
    if let Some(n) = neighbours.px {
        for v in 0..grid_size {
            for u in 0..grid_size {
                border_px[v * grid_size + u] =
                    get_representative_voxel(n, 0, u * lod, v * lod, lod);
            }
        }
    }
    if let Some(n) = neighbours.nx {
        for v in 0..grid_size {
            for u in 0..grid_size {
                border_nx[v * grid_size + u] =
                    get_representative_voxel(n, 31 - (lod - 1), u * lod, v * lod, lod);
            }
        }
    }
    if let Some(n) = neighbours.py {
        for v in 0..grid_size {
            for u in 0..grid_size {
                border_py[v * grid_size + u] =
                    get_representative_voxel(n, u * lod, 0, v * lod, lod);
            }
        }
    }
    if let Some(n) = neighbours.ny {
        for v in 0..grid_size {
            for u in 0..grid_size {
                border_ny[v * grid_size + u] =
                    get_representative_voxel(n, u * lod, 31 - (lod - 1), v * lod, lod);
            }
        }
    }
    if let Some(n) = neighbours.pz {
        for v in 0..grid_size {
            for u in 0..grid_size {
                border_pz[v * grid_size + u] =
                    get_representative_voxel(n, u * lod, v * lod, 0, lod);
            }
        }
    }
    if let Some(n) = neighbours.nz {
        for v in 0..grid_size {
            for u in 0..grid_size {
                border_nz[v * grid_size + u] =
                    get_representative_voxel(n, u * lod, v * lod, 31 - (lod - 1), lod);
            }
        }
    }

    let max_faces = grid_size * grid_size * grid_size * 6;
    let mut vertices: Vec<VoxelVertex> = Vec::with_capacity(max_faces * 4);
    let mut indices: Vec<u32> = Vec::with_capacity(max_faces * 6);

    // get if the neighbour of the current voxel is solid
    let neighbour_solid = |face: usize, gx: usize, gy: usize, gz: usize| -> bool {
        match face {
            0 => {
                // +X
                if gx + 1 < grid_size {
                    grid[gz * grid_size * grid_size + gy * grid_size + gx + 1] != 0
                } else {
                    border_px[gz * grid_size + gy] != 0
                }
            }
            1 => {
                // -X
                if gx > 0 {
                    grid[gz * grid_size * grid_size + gy * grid_size + gx - 1] != 0
                } else {
                    border_nx[gz * grid_size + gy] != 0
                }
            }
            2 => {
                // +Y
                if gy + 1 < grid_size {
                    grid[gz * grid_size * grid_size + (gy + 1) * grid_size + gx] != 0
                } else {
                    border_py[gz * grid_size + gx] != 0
                }
            }
            3 => {
                // -Y
                if gy > 0 {
                    grid[gz * grid_size * grid_size + (gy - 1) * grid_size + gx] != 0
                } else {
                    border_ny[gz * grid_size + gx] != 0
                }
            }
            4 => {
                // +Z
                if gz + 1 < grid_size {
                    grid[(gz + 1) * grid_size * grid_size + gy * grid_size + gx] != 0
                } else {
                    border_pz[gy * grid_size + gx] != 0
                }
            }
            _ => {
                // -Z
                if gz > 0 {
                    grid[(gz - 1) * grid_size * grid_size + gy * grid_size + gx] != 0
                } else {
                    border_nz[gy * grid_size + gx] != 0
                }
            }
        }
    };

    // for each voxel
    for gz in 0..grid_size {
        for gy in 0..grid_size {
            let row_base = gz * grid_size * grid_size + gy * grid_size;
            for gx in 0..grid_size {
                let id = grid[row_base + gx];
                if id == 0 {
                    continue; // skip air immediately
                }

                let vx = (gx * lod) as u32;
                let vy = (gy * lod) as u32;
                let vz = (gz * lod) as u32;

                let voxel_def = &registry.defs[id as usize];

                // render each face
                for face in 0..6usize {
                    // if the neighbouring face is solid skip
                    if neighbour_solid(face, gx, gy, gz) {
                        continue;
                    }

                    let texture_id = voxel_def.textures.get_for_face(face as u8, vx, vy, vz);

                    let x = vx as u8;
                    let y = vy as u8;
                    let z = vz as u8;
                    let l = lod as u8;

                    let corners: [[u8; 3]; 4] = match face {
                        0 => [
                            [x + l, y, z],
                            [x + l, y + l, z],
                            [x + l, y + l, z + l],
                            [x + l, y, z + l],
                        ],
                        1 => [[x, y, z + l], [x, y + l, z + l], [x, y + l, z], [x, y, z]],
                        2 => [
                            [x, y + l, z + l],
                            [x + l, y + l, z + l],
                            [x + l, y + l, z],
                            [x, y + l, z],
                        ],
                        3 => [[x, y, z], [x + l, y, z], [x + l, y, z + l], [x, y, z + l]],
                        4 => [
                            [x + l, y, z + l],
                            [x + l, y + l, z + l],
                            [x, y + l, z + l],
                            [x, y, z + l],
                        ],
                        _ => [[x, y, z], [x, y + l, z], [x + l, y + l, z], [x + l, y, z]],
                    };

                    let base = vertices.len() as u32;

                    // push to the buffers
                    vertices.push(VoxelVertex::pack(
                        corners[0][0],
                        corners[0][1],
                        corners[0][2],
                        face as u8,
                        0,
                        0,
                        texture_id as u16,
                        1,
                        1,
                    ));
                    vertices.push(VoxelVertex::pack(
                        corners[1][0],
                        corners[1][1],
                        corners[1][2],
                        face as u8,
                        1,
                        0,
                        texture_id as u16,
                        1,
                        1,
                    ));
                    vertices.push(VoxelVertex::pack(
                        corners[2][0],
                        corners[2][1],
                        corners[2][2],
                        face as u8,
                        1,
                        1,
                        texture_id as u16,
                        1,
                        1,
                    ));
                    vertices.push(VoxelVertex::pack(
                        corners[3][0],
                        corners[3][1],
                        corners[3][2],
                        face as u8,
                        0,
                        1,
                        texture_id as u16,
                        1,
                        1,
                    ));

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
