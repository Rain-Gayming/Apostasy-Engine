use cgmath::Vector3;

use crate::game::{
    game_constants::CHUNK_SIZE,
    world::voxel::{Voxel, VoxelType},
};

#[derive(Clone)]
pub struct Chunk {
    pub position: Vector3<i32>,
    pub voxels: Vec<Voxel>,
}

pub fn generate_chunk(position: Vector3<i32>) -> Chunk {
    let mut voxels: Vec<Voxel> = Vec::with_capacity(CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE);

    for _x in 0..CHUNK_SIZE {
        for _y in 0..CHUNK_SIZE {
            for _z in 0..CHUNK_SIZE {
                voxels.push(Voxel {
                    voxel_type: VoxelType::Stone,
                });
            }
        }
    }

    Chunk { voxels, position }
}
