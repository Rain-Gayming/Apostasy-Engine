use crate::app::engine::ecs::component::Component;
use cgmath::Quaternion;

pub struct RotationComponent {
    pub rotation: Quaternion<f32>,
    pub pitch: f32,
    pub yaw: f32,
    pub roll: f32,
}
impl Component for RotationComponent {}
