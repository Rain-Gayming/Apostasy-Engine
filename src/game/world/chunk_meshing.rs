use crate::app::engine::renderer::voxel_vertex::VoxelVertex;

pub enum Direction {
    Up,
    Down,
    Left,
    Right,
    Front,
    Back,
}

pub struct ChunkQuad {
    pub direction: Direction,
    pub vertices: Vec<VoxelVertex>,
    pub indices: Vec<u16>,
}

#[derive(Default)]
pub struct ChunkMesh {
    pub indices: Vec<u32>,
    pub vertices: Vec<VoxelVertex>,
}
