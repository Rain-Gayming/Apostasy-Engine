use cgmath::Vector3;

use crate::game::{
    game_constants::CHUNK_SIZE,
    world::voxel::{Voxel, VoxelType},
};

#[derive(Default)]
pub struct Chunk {
    pub voxels: Vec<Voxel>,
}

pub fn generate_chunk(position: Vector3<i32>) -> Chunk {
    let mut chunk = Chunk::default();

    for _x in 0..CHUNK_SIZE {
        for _y in 0..CHUNK_SIZE {
            for _z in 0..CHUNK_SIZE {
                chunk.voxels.push(Voxel {
                    voxel_type: VoxelType::Stone,
                });
            }
        }
    }

    chunk
}
