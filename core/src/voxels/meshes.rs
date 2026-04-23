use anyhow::Result;
use apostasy_macros::{Component, Tag};
use ash::vk::Buffer;
use ash::vk::{self, CommandPool, DeviceMemory};

use crate::objects::world::World;
use crate::rendering::shared::model::GpuMesh;
use crate::rendering::shared::vertex::VertexDefinition;
use crate::rendering::vulkan::rendering_context::VulkanRenderingContext;
use crate::utils::flatten::flatten;
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

pub fn remesh_chunks(
    world: &mut World,
    ctx: &VulkanRenderingContext,
    command_pool: CommandPool,
) -> Result<()> {
    let registry = world
        .get_resource::<VoxelRegistry>()
        .expect("No VoxelRegistry resource")
        .clone();

    for object in world.get_objects_with_tag_mut::<NeedsRemeshing>() {
        let Ok(chunk) = object.get_component::<Chunk>() else {
            continue;
        };
        let chunk = chunk.clone();

        let (vertices, indices) = generate_mesh(&chunk, &registry);

        if vertices.is_empty() || indices.is_empty() {
            println!("Empty mesh, skipping upload");
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
pub fn generate_mesh(chunk: &Chunk, registry: &VoxelRegistry) -> (Vec<VoxelVertex>, Vec<u32>) {
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

                    // check if neighbouring LOD cell has any solid voxel
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
                        false
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
