#[derive(Debug, Clone, Copy)]

pub enum VoxelType {
    Air,
    Stone,
}

#[derive(Clone, Copy)]
pub struct Voxel {
    pub voxel_type: VoxelType,
}

impl VoxelType {
    pub fn is_solid(&self) -> bool {
        !matches!(self, VoxelType::Air)
    }
}
