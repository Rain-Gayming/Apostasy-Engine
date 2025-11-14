use cgmath::Matrix4;
use component_derive::DeriveComponent;

use crate::app::engine::ecs::component::Component;

#[derive(Clone, DeriveComponent)]
pub struct CameraComponent {
    pub far: f32,
    pub near: f32,
    pub fovy: f32,
    pub projection_matrix: Matrix4<f32>,
}
