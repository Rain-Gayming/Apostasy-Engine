use crate::app::engine::ecs::Component;
use cgmath::{Vector3, Zero};
use component_derive::DeriveComponent;

#[derive(DeriveComponent)]
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
