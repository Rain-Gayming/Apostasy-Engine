use crate::app::engine::ecs::component::Component;
use cgmath::Vector3;
use component_derive::DeriveComponent;

#[derive(DeriveComponent)]
pub struct ScaleComponent {
    pub size: Vector3<f32>,
}
