use crate::engine::ecs::entity::{ColumnReadGuard, Entity, EntityView};
use crate::engine::voxels::chunk_loader::ChunkStorage;
use crate::{self as apostasy, engine::ecs::component::Component};

use crate::engine::{
    ecs::{World, components::transform::VoxelChunkTransform},
    rendering::models::{
        model::{Mesh, MeshRenderer},
        vertex::{VertexType, VoxelVertex},
    },
    voxels::{Voxel, VoxelTypeId, voxel_registry::VoxelRegistry},
};
use apostasy_macros::{Component, update};
use cgmath::Vector3;

#[derive(Component)]
pub struct UnmeshedVoxelChunk;
#[derive(Component)]
pub struct UngeneratedVoxelChunk;

#[derive(Component, Default)]
pub struct VoxelChunk {
    pub voxels: Vec<Voxel>,
}

#[update(priority = 1)]
pub fn create_chunks(world: &mut World) {
    world
        .query()
        .include::<VoxelChunk>()
        .include::<VoxelChunkTransform>()
        .include::<UnmeshedVoxelChunk>()
        .include::<UngeneratedVoxelChunk>()
        .build()
        .run(|entity| {
            let transform = entity.get_mut::<VoxelChunkTransform>().unwrap();
            let mut chunk = entity.get_mut::<VoxelChunk>().unwrap();

            world.with_resource::<VoxelRegistry, _>(|registry| {
                let voxels = generate_chunk(registry, transform.position);
                chunk.voxels = voxels;
            });
            entity.remove(UngeneratedVoxelChunk::id());
        });
}

#[update]
pub fn generate_chunk_meshes(world: &mut World) {
    world
        .query()
        .include::<VoxelChunk>()
        .include::<VoxelChunkTransform>()
        .include::<UnmeshedVoxelChunk>()
        .exclude::<UngeneratedVoxelChunk>()
        .build()
        .run(|entity| {
            let transform = entity.get_mut::<VoxelChunkTransform>().unwrap();
            let chunk = entity.get_mut::<VoxelChunk>().unwrap();

            let chunk_pos = transform.position / CHUNK_SIZE as i32;

            // First, get neighbor entity IDs WITHOUT locking resources
            let mut neighbor_entity_ids: Vec<Option<Entity>> = Vec::new();
            world.with_resource::<ChunkStorage, _>(|storage| {
                let negative_z = chunk_pos + Vector3::new(0, 0, -1);
                let positive_z = chunk_pos + Vector3::new(0, 0, 1);
                let positive_x = chunk_pos + Vector3::new(1, 0, 0);
                let negative_x = chunk_pos + Vector3::new(-1, 0, 0);
                let negative_y = chunk_pos + Vector3::new(0, -1, 0);
                let positive_y = chunk_pos + Vector3::new(0, 1, 0);

                neighbor_entity_ids = vec![
                    storage.loaded_chunks.get(&negative_z).copied(),
                    storage.loaded_chunks.get(&positive_z).copied(),
                    storage.loaded_chunks.get(&positive_x).copied(),
                    storage.loaded_chunks.get(&negative_x).copied(),
                    storage.loaded_chunks.get(&negative_y).copied(),
                    storage.loaded_chunks.get(&positive_y).copied(),
                ];
            });

            // NOW get the entity views (resource lock is released)
            let neighbour_chunks: Vec<_> = neighbor_entity_ids
                .iter()
                .map(|entity_id| {
                    if let Some(id) = entity_id {
                        world.entity(*id).get_ref::<VoxelChunk>()
                    } else {
                        None
                    }
                })
                .collect();

            let (vertices, indices) = generate_chunk_mesh(&chunk.voxels, neighbour_chunks);

            let context = world.rendering_context.clone();

            if !vertices.is_empty() {
                let vertex_buffer = context.create_vertex_buffer(vertices.as_slice()).unwrap();
                let index_buffer = context.create_index_buffer(&indices).unwrap();

                let mesh = Mesh {
                    vertex_buffer: vertex_buffer.0,
                    vertex_buffer_memory: vertex_buffer.1,
                    index_buffer: index_buffer.0,
                    index_buffer_memory: index_buffer.1,
                    index_count: indices.len() as u32,
                    vertex_type: VertexType::Voxel,
                };

                entity.insert(MeshRenderer(mesh.clone()));
            }

            entity.remove(UnmeshedVoxelChunk::id());
        });
}

pub const CHUNK_SIZE: usize = 32;

pub fn generate_chunk(registry: &VoxelRegistry, chunk_pos: Vector3<i32>) -> Vec<Voxel> {
    let stone_id = registry
        .get_numeric_id(&VoxelTypeId::from_str("apostasy:voxel:dirt"))
        .unwrap();

    let mut voxels = Vec::new();

    // Simple generation
    for _x in 0..CHUNK_SIZE {
        for _y in 0..CHUNK_SIZE {
            for _z in 0..CHUNK_SIZE {
                let voxel = Voxel::new(stone_id);

                voxels.push(voxel);
            }
        }
    }

    voxels
}
pub fn generate_chunk_mesh(
    voxels: &Vec<Voxel>,
    neighbours: Vec<Option<&VoxelChunk>>,
) -> (Vec<VoxelVertex>, Vec<u32>) {
    let estimated_vertices = (CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE) * 6 * 4 / 4;
    let estimated_indices = (CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE) * 6 * 6 / 4;
    let mut vertices = Vec::with_capacity(estimated_vertices);
    let mut indices = Vec::with_capacity(estimated_indices);

    for x in 0..CHUNK_SIZE {
        for z in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                let voxel_index = x + y * CHUNK_SIZE + z * CHUNK_SIZE * CHUNK_SIZE;
                if voxels[voxel_index] == Voxel::EMPTY {
                    continue;
                }

                // Face 0: -Z (back)
                if z == 0 {
                    if let Some(neighbor) = neighbours[0] {
                        let neighbor_voxel = neighbor.voxels
                            [x + y * CHUNK_SIZE + (CHUNK_SIZE - 1) * CHUNK_SIZE * CHUNK_SIZE];
                        if neighbor_voxel == Voxel::EMPTY {
                            add_face_inline(
                                &mut vertices,
                                &mut indices,
                                x as u8,
                                y as u8,
                                z as u8,
                                0,
                            );
                        }
                    } else {
                        add_face_inline(&mut vertices, &mut indices, x as u8, y as u8, z as u8, 0);
                    }
                } else {
                    if voxels[x + y * CHUNK_SIZE + (z - 1) * CHUNK_SIZE * CHUNK_SIZE]
                        == Voxel::EMPTY
                    {
                        add_face_inline(&mut vertices, &mut indices, x as u8, y as u8, z as u8, 0);
                    }
                }

                // Face 1: +Z (front)
                if z == CHUNK_SIZE - 1 {
                    if let Some(neighbor) = neighbours[1] {
                        let neighbor_voxel =
                            neighbor.voxels[x + y * CHUNK_SIZE + 0 * CHUNK_SIZE * CHUNK_SIZE];
                        if neighbor_voxel == Voxel::EMPTY {
                            add_face_inline(
                                &mut vertices,
                                &mut indices,
                                x as u8,
                                y as u8,
                                z as u8,
                                1,
                            );
                        }
                    } else {
                        add_face_inline(&mut vertices, &mut indices, x as u8, y as u8, z as u8, 1);
                    }
                } else {
                    if voxels[x + y * CHUNK_SIZE + (z + 1) * CHUNK_SIZE * CHUNK_SIZE]
                        == Voxel::EMPTY
                    {
                        add_face_inline(&mut vertices, &mut indices, x as u8, y as u8, z as u8, 1);
                    }
                }

                // Face 2: +X (right)
                if x == CHUNK_SIZE - 1 {
                    if let Some(neighbor) = neighbours[2] {
                        let neighbor_voxel =
                            neighbor.voxels[0 + y * CHUNK_SIZE + z * CHUNK_SIZE * CHUNK_SIZE];
                        if neighbor_voxel == Voxel::EMPTY {
                            add_face_inline(
                                &mut vertices,
                                &mut indices,
                                x as u8,
                                y as u8,
                                z as u8,
                                2,
                            );
                        }
                    } else {
                        add_face_inline(&mut vertices, &mut indices, x as u8, y as u8, z as u8, 2);
                    }
                } else {
                    if voxels[(x + 1) + y * CHUNK_SIZE + z * CHUNK_SIZE * CHUNK_SIZE]
                        == Voxel::EMPTY
                    {
                        add_face_inline(&mut vertices, &mut indices, x as u8, y as u8, z as u8, 2);
                    }
                }

                // Face 3: -X (left)
                if x == 0 {
                    if let Some(neighbor) = neighbours[3] {
                        let neighbor_voxel = neighbor.voxels
                            [(CHUNK_SIZE - 1) + y * CHUNK_SIZE + z * CHUNK_SIZE * CHUNK_SIZE];
                        if neighbor_voxel == Voxel::EMPTY {
                            add_face_inline(
                                &mut vertices,
                                &mut indices,
                                x as u8,
                                y as u8,
                                z as u8,
                                3,
                            );
                        }
                    } else {
                        add_face_inline(&mut vertices, &mut indices, x as u8, y as u8, z as u8, 3);
                    }
                } else {
                    if voxels[(x - 1) + y * CHUNK_SIZE + z * CHUNK_SIZE * CHUNK_SIZE]
                        == Voxel::EMPTY
                    {
                        add_face_inline(&mut vertices, &mut indices, x as u8, y as u8, z as u8, 3);
                    }
                }

                // Face 4: -Y (bottom)
                if y == 0 {
                    if let Some(neighbor) = neighbours[4] {
                        let neighbor_voxel = neighbor.voxels
                            [x + (CHUNK_SIZE - 1) * CHUNK_SIZE + z * CHUNK_SIZE * CHUNK_SIZE];
                        if neighbor_voxel == Voxel::EMPTY {
                            add_face_inline(
                                &mut vertices,
                                &mut indices,
                                x as u8,
                                y as u8,
                                z as u8,
                                4,
                            );
                        }
                    } else {
                        add_face_inline(&mut vertices, &mut indices, x as u8, y as u8, z as u8, 4);
                    }
                } else {
                    if voxels[x + (y - 1) * CHUNK_SIZE + z * CHUNK_SIZE * CHUNK_SIZE]
                        == Voxel::EMPTY
                    {
                        add_face_inline(&mut vertices, &mut indices, x as u8, y as u8, z as u8, 4);
                    }
                }

                // Face 5: +Y (top)
                if y == CHUNK_SIZE - 1 {
                    if let Some(neighbor) = neighbours[5] {
                        let neighbor_voxel =
                            neighbor.voxels[x + 0 * CHUNK_SIZE + z * CHUNK_SIZE * CHUNK_SIZE];
                        if neighbor_voxel == Voxel::EMPTY {
                            add_face_inline(
                                &mut vertices,
                                &mut indices,
                                x as u8,
                                y as u8,
                                z as u8,
                                5,
                            );
                        }
                    } else {
                        add_face_inline(&mut vertices, &mut indices, x as u8, y as u8, z as u8, 5);
                    }
                } else {
                    if voxels[x + (y + 1) * CHUNK_SIZE + z * CHUNK_SIZE * CHUNK_SIZE]
                        == Voxel::EMPTY
                    {
                        add_face_inline(&mut vertices, &mut indices, x as u8, y as u8, z as u8, 5);
                    }
                }
            }
        }
    }
    (vertices, indices)
}

#[inline(always)]
pub fn add_face_inline(
    vertices: &mut Vec<VoxelVertex>,
    indices: &mut Vec<u32>,
    x: u8,
    y: u8,
    z: u8,
    face: usize,
) {
    let base_index = vertices.len() as u32;

    indices.push(CUBE_INDICES[0] + base_index);
    indices.push(CUBE_INDICES[1] + base_index);
    indices.push(CUBE_INDICES[2] + base_index);
    indices.push(CUBE_INDICES[3] + base_index);
    indices.push(CUBE_INDICES[4] + base_index);
    indices.push(CUBE_INDICES[5] + base_index);

    let face_offset = face * 4;

    for i in 0..4 {
        let vertex_position = CUBE_VERTICES[face_offset + i];
        let pos_x = (vertex_position[0] + x) as u32;
        let pos_y = (vertex_position[1] + y) as u32;
        let pos_z = (vertex_position[2] + z) as u32;
        vertices.push(VoxelVertex {
            data: ((pos_z & 63) << 12) | ((pos_y & 63) << 6) | (pos_x & 63),
        });
    }
}

static CUBE_VERTICES: [[u8; 3]; 24] = [
    // Front (Z = 0, facing -Z)
    [0, 0, 0], // 0
    [1, 0, 0], // 1
    [1, 1, 0], // 2
    [0, 1, 0], // 3
    // Back (Z = 1, facing +Z)
    [1, 0, 1], // 4
    [0, 0, 1], // 5
    [0, 1, 1], // 6
    [1, 1, 1], // 7
    // Right (X = 1, facing +X)
    [1, 0, 0], // 8
    [1, 0, 1], // 9
    [1, 1, 1], // 10
    [1, 1, 0], // 11
    // Left (X = 0, facing -X)
    [0, 0, 1], // 12
    [0, 0, 0], // 13
    [0, 1, 0], // 14
    [0, 1, 1], // 15
    // Bottom (Y = 0, facing -Y)
    [0, 0, 0], // 16
    [0, 0, 1], // 17
    [1, 0, 1], // 18
    [1, 0, 0], // 19
    // Top (Y = 1, facing +Y)
    [0, 1, 0], // 20
    [1, 1, 0], // 21
    [1, 1, 1], // 22
    [0, 1, 1], // 23
];

static CUBE_INDICES: [u32; 36] = [
    0, 1, 2, 2, 3, 0, // Front
    4, 5, 6, 6, 7, 4, // Back
    8, 9, 10, 10, 11, 8, // Right
    12, 13, 14, 14, 15, 12, // Left
    16, 17, 18, 18, 19, 16, // Bottom
    20, 21, 22, 22, 23, 20, // Top
];
//
//      (0,1,1)──────(1,1,1)
//         /│           /│
//        / │          / │
//    (0,1,0)──────(1,1,0)│
//       │  │         │  │
//       │(0,0,1)─────│─(1,0,1)
//       │ /          │ /
//       │/           │/
//    (0,0,0)──────(1,0,0)
