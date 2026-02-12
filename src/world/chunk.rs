use std::sync::Arc;

use apostasy::engine::rendering::{
    models::{model::Mesh, vertex::VoxelVertex},
    rendering_context::RenderingContext,
};

pub fn create_chunk(context: &Arc<RenderingContext>) -> Mesh {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    for vertex in CUBE_VERTICES {
        let data = ((vertex[2] & 63) << 12) | ((vertex[1] & 63) << 6) | (vertex[0] & 63);
        vertices.push(VoxelVertex { data })
    }

    for index in CUBE_INDICES {
        indices.push(index);
    }

    let vertex_buffer = context.create_vertex_buffer(vertices.as_slice()).unwrap();
    let index_buffer = context.create_index_buffer(&indices).unwrap();
    Mesh {
        vertex_buffer: vertex_buffer.0,
        vertex_buffer_memory: vertex_buffer.1,
        index_buffer: index_buffer.0,
        index_buffer_memory: index_buffer.1,
        index_count: indices.len() as u32,
        // material,
    }
}

static CUBE_VERTICES: [[u32; 3]; 8] = [
    [0, 0, 0],
    [1, 0, 0],
    [1, 1, 0],
    [0, 1, 0],
    [0, 0, 1],
    [1, 0, 1],
    [1, 1, 1],
    [0, 1, 1],
];

static CUBE_INDICES: [u32; 36] = [
    0, 1, 2, 0, 2, 3, 4, 5, 6, 4, 6, 7, 8, 9, 10, 8, 10, 11, 12, 13, 14, 12, 14, 15, 16, 17, 18,
    16, 18, 19, 20, 21, 22, 20, 22, 23,
];
