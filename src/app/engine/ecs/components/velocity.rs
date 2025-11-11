use cgmath::{Vector3, Zero};

use crate::app::engine::ecs::component::Component;

pub struct VelocityComponent {
    pub velocity: Vector3<f32>,
}
impl Default for VelocityComponent {
    fn default() -> Self {
        VelocityComponent {
            velocity: Vector3::zero(),
        }
    }
}

impl Component for VelocityComponent {}
