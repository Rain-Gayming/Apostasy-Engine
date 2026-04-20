use apostasy_macros::Component;

use crate::{
    utils::flatten::flatten,
    voxels::voxel::{VoxelDefinition, VoxelId, VoxelRegistry},
};

#[derive(Clone, Component)]
struct Chunk {
    voxels: Box<[VoxelId; 32 * 32 * 32]>,
}

impl Chunk {
    fn get_def<'a>(
        &self,
        x: u32,
        y: u32,
        z: u32,
        registry: &'a VoxelRegistry,
    ) -> &'a VoxelDefinition {
        let id = self.voxels[flatten(x, y, z, 32 * 32 * 32)];
        &registry.defs[id as usize]
    }
}
