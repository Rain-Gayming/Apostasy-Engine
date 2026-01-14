use crate::{self as apostasy, engine::ecs::components::transform::Transform};
use apostasy_macros::Component;
use cgmath::Vector3;

#[derive(Component)]
pub struct Velocity {
    pub direction: Vector3<f32>,
    pub speed: f32,
}

pub fn apply_velocity(velocity: &Velocity, transform: &mut Transform) {
    transform.position += velocity.direction * velocity.speed;
}
