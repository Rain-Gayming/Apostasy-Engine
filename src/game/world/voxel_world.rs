use std::collections::HashMap;

use cgmath::Vector3;

use crate::game::world::chunk::Chunk;

pub struct VoxelWorld {
    pub chunks_rendering: HashMap<Vector3<i32>, Chunk>,
    pub chunks_to_unload: Vec<Vector3<i32>>,
    pub chunks_to_load: HashMap<Vector3<i32>, Chunk>,
    pub chunks_loaded: HashMap<Vector3<i32>, Chunk>,
}

pub fn new_voxel_world() -> VoxelWorld {
    VoxelWorld {
        chunks_rendering: HashMap::new(),
        chunks_to_unload: Vec::new(),
        chunks_to_load: HashMap::new(),
        chunks_loaded: HashMap::new(),
    }
}
