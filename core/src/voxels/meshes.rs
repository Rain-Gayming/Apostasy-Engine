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

    const FACE_NORMALS: [[i32; 3]; 6] = [
        [1, 0, 0],  // +X
        [-1, 0, 0], // -X
        [0, 1, 0],  // +Y
        [0, -1, 0], // -Y
        [0, 0, 1],  // +Z
        [0, 0, -1], // -Z
    ];

    for z in 0..32usize {
        for y in 0..32usize {
            for x in 0..32usize {
                let id = chunk.voxels[flatten(x as u32, y as u32, z as u32, 32)];
                if id == 0 {
                    continue;
                }

                for face in 0..6usize {
                    let normal = FACE_NORMALS[face];
                    let nx = x as i32 + normal[0];
                    let ny = y as i32 + normal[1];
                    let nz = z as i32 + normal[2];

                    // check neighbour
                    let neighbour_solid =
                        if nx >= 0 && nx < 32 && ny >= 0 && ny < 32 && nz >= 0 && nz < 32 {
                            let nid = chunk.voxels[flatten(nx as u32, ny as u32, nz as u32, 32)];
                            nid != 0
                        } else {
                            false
                        };

                    if neighbour_solid {
                        continue;
                    }

                    let texture_id = registry.defs[id as usize]
                        .textures
                        .get_for_face(face as u8, x as u32, y as u32, z as u32);

                    // build the single voxel quad for this face
                    let base = vertices.len() as u32;

                    // get the 4 corners of this face
                    let corners = face_corners(x, y, z, face);

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

fn face_corners(x: usize, y: usize, z: usize, face: usize) -> [[u8; 3]; 4] {
    let x = x as u8;
    let y = y as u8;
    let z = z as u8;

    match face {
        0 => [
            // +X
            [x + 1, y, z],
            [x + 1, y + 1, z],
            [x + 1, y + 1, z + 1],
            [x + 1, y, z + 1],
        ],
        1 => [
            // -X
            [x, y, z + 1],
            [x, y + 1, z + 1],
            [x, y + 1, z],
            [x, y, z],
        ],
        2 => [
            [x, y + 1, z + 1],
            [x + 1, y + 1, z + 1],
            [x + 1, y + 1, z],
            [x, y + 1, z],
        ],
        3 => [[x, y, z], [x + 1, y, z], [x + 1, y, z + 1], [x, y, z + 1]],
        4 => [
            // +Z
            [x + 1, y, z + 1],
            [x + 1, y + 1, z + 1],
            [x, y + 1, z + 1],
            [x, y, z + 1],
        ],
        _ => [
            // -Z
            [x, y, z],
            [x, y + 1, z],
            [x + 1, y + 1, z],
            [x + 1, y, z],
        ],
    }
}
