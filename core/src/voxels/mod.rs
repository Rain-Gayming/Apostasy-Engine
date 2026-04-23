use apostasy_macros::Component;
use cgmath::{Vector3, Zero};

pub mod chunk;
pub mod chunk_loader;
pub mod meshes;
pub mod texture_atlas;
pub mod voxel;

#[derive(Component, Default, Clone, Debug)]
pub struct IsSolid(bool);

#[derive(Component, Clone, Debug)]
pub struct VoxelTransform {
    pub position: Vector3<i32>,
}

impl Default for VoxelTransform {
    fn default() -> Self {
        Self {
            position: Vector3::zero(),
        }
    }
}
