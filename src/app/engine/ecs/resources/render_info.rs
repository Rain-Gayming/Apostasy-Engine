use crate::app::engine::ecs::resource::Resource;

#[derive(Default)]
pub struct RenderInfo {
    pub view_matrix: [[f32; 4]; 4],
    pub projection_matrix: [[f32; 4]; 4],
}

impl Resource for RenderInfo {}
