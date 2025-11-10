use crate::app::engine::ecs::component::Component;
use cgmath::Vector3;

pub struct ScaleComponent {
    pub size: Vector3<f32>,
}
impl Component for ScaleComponent {}
