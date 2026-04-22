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
    pub fn pack(x: u8, y: u8, z: u8, face: u8, u: u8, v: u8, texture_id: u32) -> Self {
        let data: u64 = (x as u64)        // bits 0-5,  6 bits (0-32)
            | ((y as u64) << 6)           // bits 6-11, 6 bits
            | ((z as u64) << 12)          // bits 12-17, 6 bits
            | ((face as u64) << 18)       // bits 18-20, 3 bits
            | ((u as u64) << 21)          // bits 21-22, 2 bits
            | ((v as u64) << 23) // bits 23-24, 2 bits
            | ((texture_id as u64) << 33); // 31 bits remaining for texture id
        Self { data }
    }
}
pub fn generate_mesh(chunk: &Chunk, registry: &VoxelRegistry) -> (Vec<VoxelVertex>, Vec<u32>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    // axis 0 = X, 1 = Y, 2 = Z
    for axis in 0..3usize {
        let u = (axis + 1) % 3;
        let v = (axis + 2) % 3;

        for slice in 0..32usize {
            let mut forward_mask = [[0u16; 32]; 32];
            let mut backward_mask = [[0u16; 32]; 32];

            for ui in 0..32usize {
                for vi in 0..32usize {
                    let mut pos = [0usize; 3];
                    pos[axis] = slice;
                    pos[u] = ui;
                    pos[v] = vi;

                    let id = chunk.voxels[flatten(pos[0] as u32, pos[1] as u32, pos[2] as u32, 32)];

                    if id == 0 {
                        continue;
                    }

                    // +axis neighbour
                    if slice + 1 < 32 {
                        let mut npos = pos;
                        npos[axis] = slice + 1;
                        let nid = chunk.voxels
                            [flatten(npos[0] as u32, npos[1] as u32, npos[2] as u32, 32)];
                        if nid == 0 {
                            forward_mask[ui][vi] = id;
                        }
                    } else {
                        forward_mask[ui][vi] = id;
                    }

                    // -axis neighbour
                    if slice > 0 {
                        let mut npos = pos;
                        npos[axis] = slice - 1;
                        let nid = chunk.voxels
                            [flatten(npos[0] as u32, npos[1] as u32, npos[2] as u32, 32)];
                        if nid == 0 {
                            backward_mask[ui][vi] = id;
                        }
                    } else {
                        backward_mask[ui][vi] = id;
                    }
                }
            }

            // greedy merge each mask
            for (mask, face_dir) in [(&mut forward_mask, 0u8), (&mut backward_mask, 1u8)] {
                let face = (axis * 2) as u8 + face_dir;

                let mut ui = 0;
                while ui < 32 {
                    let mut vi = 0;
                    while vi < 32 {
                        let id = mask[ui][vi];
                        if id == 0 {
                            vi += 1;
                            continue;
                        }

                        // expand width
                        let mut width = 1;
                        while ui + width < 32 && mask[ui + width][vi] == id {
                            width += 1;
                        }

                        // expand height along vertical
                        let mut height = 1;
                        'outer: while vi + height < 32 {
                            for w in 0..width {
                                if mask[ui + w][vi + height] != id {
                                    break 'outer;
                                }
                            }
                            height += 1;
                        }

                        // clear merged region
                        for w in 0..width {
                            for h in 0..height {
                                mask[ui + w][vi + h] = 0;
                            }
                        }

                        // build quad corners
                        let s = slice as u8 + if face_dir == 0 { 1 } else { 0 };

                        // corner positions depend on which axis were on
                        let (p0, p1, p2, p3) = match face_dir {
                            0 => (
                                {
                                    let mut p = [0u8; 3];
                                    p[axis] = s;
                                    p[u] = ui as u8;
                                    p[v] = vi as u8;
                                    p
                                },
                                {
                                    let mut p = [0u8; 3];
                                    p[axis] = s;
                                    p[u] = (ui + width) as u8;
                                    p[v] = vi as u8;
                                    p
                                },
                                {
                                    let mut p = [0u8; 3];
                                    p[axis] = s;
                                    p[u] = (ui + width) as u8;
                                    p[v] = (vi + height) as u8;
                                    p
                                },
                                {
                                    let mut p = [0u8; 3];
                                    p[axis] = s;
                                    p[u] = ui as u8;
                                    p[v] = (vi + height) as u8;
                                    p
                                },
                            ),
                            _ => (
                                {
                                    let mut p = [0u8; 3];
                                    p[axis] = s;
                                    p[u] = ui as u8;
                                    p[v] = (vi + height) as u8;
                                    p
                                },
                                {
                                    let mut p = [0u8; 3];
                                    p[axis] = s;
                                    p[u] = (ui + width) as u8;
                                    p[v] = (vi + height) as u8;
                                    p
                                },
                                {
                                    let mut p = [0u8; 3];
                                    p[axis] = s;
                                    p[u] = (ui + width) as u8;
                                    p[v] = vi as u8;
                                    p
                                },
                                {
                                    let mut p = [0u8; 3];
                                    p[axis] = s;
                                    p[u] = ui as u8;
                                    p[v] = vi as u8;
                                    p
                                },
                            ),
                        };

                        let base = vertices.len() as u32;
                        let texture_id = registry.defs[id as usize].textures.get_for_face(face);

                        vertices.push(VoxelVertex::pack(
                            p0[0], p0[1], p0[2], face, 0, 0, texture_id,
                        ));
                        vertices.push(VoxelVertex::pack(
                            p1[0],
                            p1[1],
                            p1[2],
                            face,
                            width as u8,
                            0,
                            texture_id,
                        ));
                        vertices.push(VoxelVertex::pack(
                            p2[0],
                            p2[1],
                            p2[2],
                            face,
                            width as u8,
                            height as u8,
                            texture_id,
                        ));
                        vertices.push(VoxelVertex::pack(
                            p3[0],
                            p3[1],
                            p3[2],
                            face,
                            0,
                            height as u8,
                            texture_id,
                        ));
                        indices.extend_from_slice(&[
                            base,
                            base + 1,
                            base + 3,
                            base + 1,
                            base + 2,
                            base + 3,
                        ]);

                        vi += height;
                    }
                    ui += 1;
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
