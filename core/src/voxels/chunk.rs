use apostasy_macros::Component;

use crate::voxels::voxel::Voxel;

#[derive(Clone, Component)]
pub struct VoxelChunk {
    pub voxels: Vec<Voxel>,
}
