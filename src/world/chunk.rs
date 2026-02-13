use apostasy::engine::{
    ecs::{World, command::Command, components::transform::VoxelChunkTransform},
    rendering::models::{
        model::{Mesh, MeshRenderer},
        vertex::{VertexType, VoxelVertex},
    },
    voxels::{Voxel, VoxelTypeId, voxel_registry::VoxelRegistry},
};
use apostasy_macros::{Component, start};
use cgmath::Vector3;

#[derive(Component, Default)]
pub struct VoxelChunk {
    pub voxels: Vec<Voxel>,
}

#[start(priority = 0)]
pub fn create_chunks(world: &mut World) {
    world
        .query()
        .include::<VoxelChunk>()
        .include::<VoxelChunkTransform>()
        .build()
        .run(|entity| {
            let transform = entity.get_mut::<VoxelChunkTransform>().unwrap();
            let mut chunk = entity.get_mut::<VoxelChunk>().unwrap();

            world.with_resource::<VoxelRegistry, _>(|registry| {
                let voxels = generate_chunk(registry, transform.position);
                chunk.voxels = voxels;
            });

            println!("Created chunk at {:?}", transform.position);
            println!("Chunk contains {} voxels", chunk.voxels.len());

            let (vertices, indices) = generate_chunk_mesh(&chunk.voxels);

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
                    // material,
                };

                entity.insert(MeshRenderer(mesh.clone()));
            }
        });
}

const CHUNK_SIZE: usize = 32;

pub fn generate_chunk(registry: &VoxelRegistry, chunk_pos: Vector3<i32>) -> Vec<Voxel> {
    let stone_id = registry
        .get_numeric_id(&VoxelTypeId::from_str("apostasy:voxel:dirt"))
        .unwrap();

    let mut voxels = Vec::new();

    // Simple generation
    for x in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let voxel = Voxel::new(stone_id);

                voxels.push(voxel);
            }
        }
    }

    voxels
}

pub fn generate_chunk_mesh(voxels: &Vec<Voxel>) -> (Vec<VoxelVertex>, Vec<u32>) {
    let mut vertices: Vec<VoxelVertex> = Vec::new();
    let mut indices = Vec::new();
    for x in 0..CHUNK_SIZE {
        for z in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                let voxel_index = x + y * CHUNK_SIZE + z * CHUNK_SIZE * CHUNK_SIZE;

                if voxels[voxel_index] != Voxel::EMPTY {
                    for face in 0..6 {
                        match face {
                            0 => {
                                if z != 0
                                    && voxels
                                        [x + y * CHUNK_SIZE + (z - 1) * CHUNK_SIZE * CHUNK_SIZE]
                                        == Voxel::EMPTY
                                    || z == 0
                                {
                                    add_face(
                                        &mut vertices,
                                        &mut indices,
                                        x as u8,
                                        y as u8,
                                        z as u8,
                                        face,
                                    );
                                }
                            }
                            1 => {
                                if z != CHUNK_SIZE - 1
                                    && voxels
                                        [x + y * CHUNK_SIZE + (z + 1) * CHUNK_SIZE * CHUNK_SIZE]
                                        == Voxel::EMPTY
                                    || z == CHUNK_SIZE - 1
                                {
                                    add_face(
                                        &mut vertices,
                                        &mut indices,
                                        x as u8,
                                        y as u8,
                                        z as u8,
                                        face,
                                    );
                                }
                            }

                            2 => {
                                if x != CHUNK_SIZE - 1
                                    && voxels
                                        [(x + 1) + y * CHUNK_SIZE + z * CHUNK_SIZE * CHUNK_SIZE]
                                        == Voxel::EMPTY
                                    || x == CHUNK_SIZE - 1
                                {
                                    add_face(
                                        &mut vertices,
                                        &mut indices,
                                        x as u8,
                                        y as u8,
                                        z as u8,
                                        face,
                                    );
                                }
                            }
                            3 => {
                                if x != 0
                                    && voxels
                                        [(x - 1) + y * CHUNK_SIZE + z * CHUNK_SIZE * CHUNK_SIZE]
                                        == Voxel::EMPTY
                                    || x == 0
                                {
                                    add_face(
                                        &mut vertices,
                                        &mut indices,
                                        x as u8,
                                        y as u8,
                                        z as u8,
                                        face,
                                    );
                                }
                            }
                            4 => {
                                if y != 0
                                    && voxels
                                        [x + (y - 1) * CHUNK_SIZE + z * CHUNK_SIZE * CHUNK_SIZE]
                                        == Voxel::EMPTY
                                {
                                    add_face(
                                        &mut vertices,
                                        &mut indices,
                                        x as u8,
                                        y as u8,
                                        z as u8,
                                        face,
                                    );
                                }
                            }
                            5 => {
                                if y != CHUNK_SIZE - 1
                                    && voxels
                                        [x + (y + 1) * CHUNK_SIZE + z * CHUNK_SIZE * CHUNK_SIZE]
                                        == Voxel::EMPTY
                                    || y == CHUNK_SIZE - 1
                                {
                                    add_face(
                                        &mut vertices,
                                        &mut indices,
                                        x as u8,
                                        y as u8,
                                        z as u8,
                                        face,
                                    );
                                }
                            }

                            x => panic!("Invalid face: {}", x),
                        }
                    }
                }
            }
        }
    }

    (vertices, indices)
}

#[allow(clippy::needless_range_loop)]
pub fn add_face(
    vertices: &mut Vec<VoxelVertex>,
    indices: &mut Vec<u32>,
    x: u8,
    y: u8,
    z: u8,
    face: usize,
) {
    for offset in 0..6 {
        indices.push(CUBE_INDICES[offset] + vertices.len() as u32);
    }

    for vertex in 0..4 {
        let vertex_position = CUBE_VERTICES[face * 4 + vertex];
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
