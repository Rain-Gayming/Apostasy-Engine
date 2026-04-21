use apostasy_macros::Component;

pub mod chunk;
pub mod meshes;
pub mod voxel;

#[derive(Component, Default, Clone, Debug)]
pub struct IsSolid(bool);
