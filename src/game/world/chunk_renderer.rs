use cgmath::Vector3;

use crate::{
    app::engine::renderer::{
        create_vertex_buffer_from_data,
        voxel_vertex::{VoxelVertex, CUBEMESH, CUBE_INDICES},
        Renderer,
    },
    game::game_constants::CHUNK_SIZE,
};

pub fn render_test_chunk(position: Vector3<i32>, renderer: &mut Renderer) {
    let mut vertex_data: Vec<VoxelVertex> = Vec::new();
    let mut index_data: Vec<u8> = Vec::new();

    // for x in 0..CHUNK_SIZE {
    //     for y in 0..CHUNK_SIZE {
    //         for z in 0..CHUNK_SIZE {
    //             for vertex in CUBEMESH.iter() {
    //                 let position = [vertex[0] + x, vertex[1] + y, vertex[2] + z];
    //                 vertex_data.push(VoxelVertex { position });
    //             }
    //             for index in CUBE_INDICES.iter() {
    //                 index_data.push(*index);
    //             }
    //         }
    //     }
    // }
    for vertex in CUBEMESH.into_iter() {
        vertex_data.push(VoxelVertex { position: vertex });
        println!("{vertex:?}");
    }

    for index in CUBE_INDICES.iter() {
        index_data.push(*index);
    }
    create_vertex_buffer_from_data(renderer, vertex_data, index_data);
}
