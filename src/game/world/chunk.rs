use std::collections::HashMap;

use cgmath::Vector3;

use crate::game::world::voxel::Voxel;

pub struct Chunk {
    pub voxels: HashMap<Vector3<u8>, Voxel>,
}
