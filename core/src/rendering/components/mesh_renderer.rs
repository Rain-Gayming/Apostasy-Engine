use apostasy_macros::Component;

use crate::rendering::shared::model::GpuMesh;

#[derive(Component, Clone)]
pub struct MeshRenderer {
    pub mesh: GpuMesh,
}
