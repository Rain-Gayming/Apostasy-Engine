use std::collections::HashMap;

use cgmath::Vector3;

use crate::game::world::chunk::Chunk;

pub struct VoxelWorld {
    pub chunks: HashMap<Vector3<i32>, Chunk>,
    pub chunks_to_unmesh: Vec<Vector3<i32>>,
}

pub fn new_voxel_world() -> VoxelWorld {
    VoxelWorld {
        chunks: HashMap::new(),
        chunks_to_unmesh: Vec::new(),
    }
}
