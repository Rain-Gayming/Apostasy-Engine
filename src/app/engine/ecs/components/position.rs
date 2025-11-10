use cgmath::Vector3;

use crate::app::engine::ecs::component::Component;

pub struct PositionComponent {
    pub position: Vector3<f32>,
}
impl Component for PositionComponent {}
