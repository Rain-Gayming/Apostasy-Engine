use std::collections::HashMap;

use cgmath::{Vector3, Zero};

use crate::{
    app::engine::renderer::Renderer,
    game::world::{chunk::Chunk, chunk_renderer::render_test_chunk},
};

pub struct VoxelWorld {
    pub chunks: HashMap<Vector3<i32>, Chunk>,
}

pub fn new_voxel_world() -> VoxelWorld {
    VoxelWorld {
        chunks: HashMap::new(),
    }
}
