use cgmath::Vector3;

use crate::{
    app::engine::renderer::{
        create_vertex_buffer_from_data,
        voxel_vertex::{VoxelVertex, CUBEMESH, CUBE_INDICES},
        Renderer,
    },
    game::{
        game_constants::{CHUNK_SIZE, CHUNK_SIZE_MINUS_ONE},
        world::voxel::{Voxel, VoxelType},
    },
};

pub fn render_test_chunk(position: Vector3<i32>, renderer: &mut Renderer) {
    let mut vertex_data: Vec<VoxelVertex> = Vec::new();
    let mut index_data: Vec<u16> = Vec::new();

    let mut voxels: Vec<Voxel> = Vec::with_capacity(CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE);

    for _x in 0..CHUNK_SIZE {
        for _y in 0..CHUNK_SIZE {
            for _z in 0..CHUNK_SIZE {
                voxels.push(Voxel {
                    voxel_type: VoxelType::Stone,
                });
            }
        }
    }

    for x in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                for _ in 0..6 {
                    if z != 0
                        && !voxels[x + y * CHUNK_SIZE + (z - 1) * CHUNK_SIZE * CHUNK_SIZE]
                            .voxel_type
                            .is_solid()
                        || z == 0
                    {
                        generate_voxel_face(
                            &mut vertex_data,
                            &mut index_data,
                            0,
                            [x as u8, y as u8, z as u8],
                        );
                    }

                    if z != CHUNK_SIZE_MINUS_ONE
                        && !voxels[x + y * CHUNK_SIZE + (z + 1) * CHUNK_SIZE * CHUNK_SIZE]
                            .voxel_type
                            .is_solid()
                        || z == CHUNK_SIZE_MINUS_ONE
                    {
                        generate_voxel_face(
                            &mut vertex_data,
                            &mut index_data,
                            1,
                            [x as u8, y as u8, z as u8],
                        );
                    }
                    if x != CHUNK_SIZE_MINUS_ONE
                        && !voxels[(x + 1) + y * CHUNK_SIZE + z * CHUNK_SIZE * CHUNK_SIZE]
                            .voxel_type
                            .is_solid()
                        || x == CHUNK_SIZE_MINUS_ONE
                    {
                        generate_voxel_face(
                            &mut vertex_data,
                            &mut index_data,
                            2,
                            [x as u8, y as u8, z as u8],
                        );
                    }
                    if x != 0
                        && !voxels[(x - 1) + y * CHUNK_SIZE + z * CHUNK_SIZE * CHUNK_SIZE]
                            .voxel_type
                            .is_solid()
                        || x == 0
                    {
                        generate_voxel_face(
                            &mut vertex_data,
                            &mut index_data,
                            3,
                            [x as u8, y as u8, z as u8],
                        );
                    }
                    if y != 0
                        && !voxels[x + (y - 1) * CHUNK_SIZE + z * CHUNK_SIZE * CHUNK_SIZE]
                            .voxel_type
                            .is_solid()
                        || y == 0
                    {
                        generate_voxel_face(
                            &mut vertex_data,
                            &mut index_data,
                            4,
                            [x as u8, y as u8, z as u8],
                        );
                    }
                    if y != CHUNK_SIZE_MINUS_ONE
                        && !voxels[x + (y + 1) * CHUNK_SIZE + z * CHUNK_SIZE * CHUNK_SIZE]
                            .voxel_type
                            .is_solid()
                        || y == CHUNK_SIZE_MINUS_ONE
                    {
                        generate_voxel_face(
                            &mut vertex_data,
                            &mut index_data,
                            5,
                            [x as u8, y as u8, z as u8],
                        );
                    }
                }
            }
        }
    }

    println!("vertex count: {}", vertex_data.len());
    println!("index count: {}", index_data.len());
    println!("voxel count: {}", voxels.len());

    if !vertex_data.is_empty() && !index_data.is_empty() {
        create_vertex_buffer_from_data(renderer, vertex_data, index_data, position);
    } else {
        println!("vertex and index buffers are empty");
    }
}

pub fn generate_voxel_face(
    vertex_data: &mut Vec<VoxelVertex>,
    index_data: &mut Vec<u16>,
    face: u8,
    position: [u8; 3],
) {
    for index in CUBE_INDICES.into_iter().take(6) {
        index_data.push(index + vertex_data.len() as u16);
    }
    for vertex in 0..4 {
        let base_position = CUBEMESH[face as usize * 4 + vertex];
        let position_x = base_position[0] + position[0];
        let position_y = base_position[1] + position[1];
        let position_z = base_position[2] + position[2];
        let position = [position_x, position_y, position_z];
        vertex_data.push(VoxelVertex { position });
    }
}
