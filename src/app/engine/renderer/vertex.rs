pub trait Vertex {}

#[derive(Debug, Clone, Copy)]
pub struct VoxelVertex {
    pub data: u32,
}
impl Vertex for VoxelVertex {}

#[derive(Debug, Clone, Copy)]
pub struct ModelVertex {
    pub position: [f32; 3],
}
impl Vertex for ModelVertex {}
