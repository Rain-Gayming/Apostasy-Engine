use crate::app::engine::ecs::component::Component;
use cgmath::Quaternion;

use component_derive::DeriveComponent;
#[derive(DeriveComponent)]
pub struct RotationComponent {
    pub rotation: Quaternion<f32>,
    pub pitch: f32,
    pub yaw: f32,
    pub roll: f32,
}
