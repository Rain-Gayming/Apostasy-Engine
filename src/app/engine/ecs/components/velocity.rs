use cgmath::Vector3;

use crate::app::engine::ecs::component::Component;

pub struct VelocityComponent {
    pub velocity: Vector3<f32>,
}

impl Component for VelocityComponent {}
