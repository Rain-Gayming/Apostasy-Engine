use cgmath::{Vector3, Zero};
use component_derive::DeriveComponent;

use crate::app::engine::ecs::component::Component;

#[derive(DeriveComponent)]
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
