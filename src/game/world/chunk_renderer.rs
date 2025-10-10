use ash::vk::BufferCreateInfo;
use cgmath::Vector3;

use crate::{
    app::engine::renderer::{
        self, create_vertex_buffer_from_data,
        voxel_vertex::{VoxelVertex, CUBEMESH},
        Renderer,
    },
    game::game_constants::CHUNK_SIZE,
};

pub fn render_test_chunk(position: Vector3<i32>, renderer: &mut Renderer) {
    let mut vertex_data: Vec<VoxelVertex> = Vec::new();

    for x in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                for vertex in CUBEMESH.iter() {
                    let position = [vertex[0] + x, vertex[1] + y, vertex[2] + z];
                    vertex_data.push(VoxelVertex { position });
                }
            }
        }
    }

    let vertex_buffer_info = BufferCreateInfo {
        size: (std::mem::size_of::<VoxelVertex>() * vertex_data.len()) as u64,
        usage: ash::vk::BufferUsageFlags::VERTEX_BUFFER,
        sharing_mode: ash::vk::SharingMode::EXCLUSIVE,
        ..Default::default()
    };
    create_vertex_buffer_from_data(vertex_buffer_info, renderer, vertex_data.len());
}
