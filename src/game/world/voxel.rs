#[derive(Debug)]
pub enum VoxelType {
    Air,
    Stone,
}

pub struct Voxel {
    pub voxel_type: VoxelType,
}

impl VoxelType {
    pub fn is_solid(&self) -> bool {
        match self {
            VoxelType::Air => false,
            _ => true,
        }
    }
}
