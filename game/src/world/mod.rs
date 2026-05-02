use apostasy_macros::Tag;

pub mod chunk_loader;
pub mod generation;
pub mod raycast;

#[derive(Tag, Clone)]
pub struct VoxelOutline;
