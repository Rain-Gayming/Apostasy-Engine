use anyhow::Result;
use apostasy_macros::{Component, Tag};
use ash::vk::Buffer;
use ash::vk::{self, CommandPool, DeviceMemory};

use crate::objects::world::World;
use crate::rendering::shared::model::GpuMesh;
use crate::rendering::shared::vertex::VertexDefinition;
use crate::rendering::vulkan::rendering_context::VulkanRenderingContext;
use crate::utils::flatten::flatten;
use crate::voxels::IsSolid;
use crate::voxels::chunk::Chunk;
use crate::voxels::voxel::VoxelRegistry;

// Face order: +X, -X, +Y, -Y, +Z, -Z
const FACE_NORMALS: [[i32; 3]; 6] = [
    [1, 0, 0],
    [-1, 0, 0],
    [0, 1, 0],
    [0, -1, 0],
    [0, 0, 1],
    [0, 0, -1],
];

const FACE_VERTICES: [[[u8; 3]; 4]; 6] = [
    [[1, 0, 0], [1, 1, 0], [1, 1, 1], [1, 0, 1]], // +X
    [[0, 0, 1], [0, 1, 1], [0, 1, 0], [0, 0, 0]], // -X
    [[0, 1, 0], [0, 1, 1], [1, 1, 1], [1, 1, 0]], // +Y
    [[0, 0, 1], [0, 0, 0], [1, 0, 0], [1, 0, 1]], // -Y
    [[1, 0, 1], [1, 1, 1], [0, 1, 1], [0, 0, 1]], // +Z
    [[0, 0, 0], [0, 1, 0], [1, 1, 0], [1, 0, 0]], // -Z
];

const FACE_UVS: [[u8; 2]; 4] = [[0, 0], [0, 1], [1, 1], [1, 0]];

impl VoxelVertex {
    pub fn pack(x: u8, y: u8, z: u8, face: u8, u: u8, v: u8) -> Self {
        let data: u64 = (x as u64)
            | ((y as u64) << 5)
            | ((z as u64) << 10)
            | ((face as u64) << 15)
            | ((u as u64) << 18)
            | ((v as u64) << 20);
        Self { data }
    }
}

pub fn generate_mesh(chunk: &Chunk, registry: &VoxelRegistry) -> (Vec<VoxelVertex>, Vec<u32>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    for z in 0..32u8 {
        for y in 0..32u8 {
            for x in 0..32u8 {
                let id = chunk.voxels[flatten(x as u32, y as u32, z as u32, 32)];

                let def = &registry.defs[id as usize];

                for (face, normal) in FACE_NORMALS.iter().enumerate() {
                    let nx = x as i32 + normal[0];
                    let ny = y as i32 + normal[1];
                    let nz = z as i32 + normal[2];

                    let neighbour_solid =
                        if nx >= 0 && nx < 32 && ny >= 0 && ny < 32 && nz >= 0 && nz < 32 {
                            let nid = chunk.voxels[flatten(nx as u32, ny as u32, nz as u32, 32)];
                            if nid == 0 {
                                false
                            } else {
                                registry.defs[nid as usize].has_component::<IsSolid>()
                            }
                        } else {
                            false
                        };

                    if neighbour_solid {
                        continue;
                    }

                    let base = vertices.len() as u32;
                    for (i, corner) in FACE_VERTICES[face].iter().enumerate() {
                        vertices.push(VoxelVertex::pack(
                            x + corner[0],
                            y + corner[1],
                            z + corner[2],
                            face as u8,
                            FACE_UVS[i][0],
                            FACE_UVS[i][1],
                        ));
                    }

                    indices.extend_from_slice(&[
                        base,
                        base + 1,
                        base + 2,
                        base,
                        base + 2,
                        base + 3,
                    ]);
                }
            }
        }
    }

    (vertices, indices)
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct VoxelVertex {
    pub data: u64,
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
