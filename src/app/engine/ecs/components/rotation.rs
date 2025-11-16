use crate::app::engine::ecs::component::Component;
use cgmath::{Deg, Euler, Quaternion};

use component_derive::DeriveComponent;
#[derive(Clone, DeriveComponent)]
pub struct RotationComponent {
    pub rotation: Quaternion<f32>,
    pub pitch: f32,
    pub yaw: f32,
    pub roll: f32,
}

pub fn rotate_component(rotation_component: &mut RotationComponent) {
    rotation_component.rotation = Quaternion::from(Euler {
        x: Deg(rotation_component.pitch),
        y: Deg(rotation_component.yaw),
        z: Deg(rotation_component.roll),
    });
}
