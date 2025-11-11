use cgmath::Vector3;
use cgmath::Zero;

use crate::app::engine::ecs::component::Component;

pub struct PositionComponent {
    pub position: Vector3<f32>,
}
impl Default for PositionComponent {
    fn default() -> Self {
        PositionComponent {
            position: Vector3::zero(),
        }
    }
}
impl Component for PositionComponent {}
