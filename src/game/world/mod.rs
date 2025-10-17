use crate::game::world::voxel_world::{new_voxel_world, VoxelWorld};

pub mod chunk;
pub mod chunk_generator;
pub mod chunk_renderer;
pub mod voxel;
pub mod voxel_world;

pub struct World {
    pub voxel_world: VoxelWorld,
}

pub fn new_world() -> World {
    World {
        voxel_world: new_voxel_world(),
    }
}
