use std::sync::Arc;

use apostasy::engine::rendering::{
    models::{
        model::Mesh,
        vertex::{VertexType, VoxelVertex},
    },
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
        vertex_type: VertexType::Voxel,
        // material,
    }
}
static CUBE_VERTICES: [[u32; 3]; 8] = [
    [0, 0, 0], // 0
    [1, 0, 0], // 1
    [1, 1, 0], // 2
    [0, 1, 0], // 3
    [0, 0, 1], // 4
    [1, 0, 1], // 5
    [1, 1, 1], // 6
    [0, 1, 1], // 7
];

static CUBE_INDICES: [u32; 36] = [
    // Front face (z = 0) - looking at -Z
    0, 2, 1, 0, 3, 2, // Back face (z = 1) - looking at +Z
    4, 5, 6, 4, 6, 7, // Left face (x = 0) - looking at -X
    0, 4, 7, 0, 7, 3, // Right face (x = 1) - looking at +X
    1, 2, 6, 1, 6, 5, // Bottom face (y = 0) - looking at -Y
    0, 1, 5, 0, 5, 4, // Top face (y = 1) - looking at +Y
    3, 7, 6, 3, 6, 2,
];
