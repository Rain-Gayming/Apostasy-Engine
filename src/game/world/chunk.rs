use cgmath::{Vector3, Zero};

use crate::game::{
    game_constants::CHUNK_SIZE,
    world::{
        chunk_renderer::ChunkMesh,
        voxel::{Voxel, VoxelType},
    },
};

#[derive(Clone)]
pub struct Chunk {
    pub position: Vector3<i32>,
    pub voxels: Vec<Voxel>,
    pub mesh: ChunkMesh,
}
pub struct ChunkSend {
    chunk: Chunk,
}

impl Default for Chunk {
    fn default() -> Self {
        Chunk {
            position: Vector3::zero(),
            voxels: Vec::new(),
            mesh: ChunkMesh::default(),
        }
    }
}

pub fn generate_chunk(position: Vector3<i32>, chunk: &mut Chunk) {
    chunk.position = position;

    for _x in 0..CHUNK_SIZE {
        for _y in 0..CHUNK_SIZE {
            for _z in 0..CHUNK_SIZE {
                chunk.voxels.push(Voxel {
                    voxel_type: VoxelType::Stone,
                });
            }
        }
    }
}
