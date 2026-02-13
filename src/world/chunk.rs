use std::sync::Arc;

use apostasy::engine::rendering::{
    models::{
        model::Mesh,
        vertex::{VertexType, VoxelVertex},
    },
    rendering_context::RenderingContext,
};
use apostasy_macros::Component;

#[derive(Component)]
pub struct VoxelChunk {}

pub fn create_chunk(context: &Arc<RenderingContext>) -> Mesh {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    add_face(&mut vertices, &mut indices, 0, 0, 0, 0);
    add_face(&mut vertices, &mut indices, 0, 0, 0, 1);
    add_face(&mut vertices, &mut indices, 0, 0, 0, 2);
    add_face(&mut vertices, &mut indices, 0, 0, 0, 3);
    add_face(&mut vertices, &mut indices, 0, 0, 0, 4);
    add_face(&mut vertices, &mut indices, 0, 0, 0, 5);

    let vertex_buffer = context.create_vertex_buffer(vertices.as_slice()).unwrap();
    let index_buffer = context.create_index_buffer(&indices).unwrap();
    Mesh {
        vertex_buffer: vertex_buffer.0,
        vertex_buffer_memory: vertex_buffer.1,
        index_buffer: index_buffer.0,
        index_buffer_memory: index_buffer.1,
        index_count: indices.len() as u32,
        vertex_type: VertexType::Voxel,
        // material,
    }
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
    // front
    [1, 0, 0], // 0
    [0, 0, 0], // 1
    [0, 1, 0], // 2
    [1, 1, 0], // 3
    // back
    [0, 0, 1], // 4
    [1, 0, 1], // 5
    [1, 1, 1], // 6
    [0, 1, 1], // 7
    // left
    [1, 1, 0], // 8
    [1, 1, 1], // 9
    [1, 0, 1], // 10
    [1, 0, 0], // 11
    // right
    [0, 1, 0], // 12
    [0, 0, 0], // 13
    [0, 0, 1], // 14
    [0, 1, 1], // 15
    //bottom
    [1, 0, 0], // 16
    [0, 0, 0], // 17
    [1, 0, 1], // 18
    [0, 0, 1], // 19
    // top
    [0, 1, 0], // 20
    [0, 1, 1], // 21
    [1, 1, 1], // 22
    [1, 1, 0], // 23
];

static CUBE_INDICES: [u32; 36] = [
    0, 1, 2, 2, 3, 0, // front
    4, 5, 6, 6, 7, 4, // back
    8, 9, 10, 10, 11, 8, // left
    12, 13, 14, 14, 15, 12, // right
    16, 17, 18, 18, 19, 16, // bottom
    20, 21, 22, 22, 23, 20, // top
];
