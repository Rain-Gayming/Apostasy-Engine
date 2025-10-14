use std::collections::HashMap;

use cgmath::Vector3;

use crate::game::world::chunk::Chunk;

pub struct VoxelWorld {
    pub chunks: HashMap<Vector3<i32>, Chunk>,
}

pub fn new_voxel_world() -> VoxelWorld {
    VoxelWorld {
        chunks: HashMap::new(),
    }
}
