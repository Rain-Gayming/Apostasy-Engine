use crate::engine::editor::inspectable::Inspectable;
use crate::engine::nodes::World;
use crate::engine::physics::collider::{Collider, ColliderShape};
use crate::engine::nodes::components::transform::Transform;
use crate::engine::physics::velocity::Velocity;
use crate::{self as apostasy};
use apostasy_macros::InspectValue;
use apostasy_macros::{Component, Inspectable, SerializableComponent, fixed_update};
use cgmath::{InnerSpace, Quaternion, Rotation3, Vector3, Zero};
use serde::{Deserialize, Serialize};

#[derive(
    Component, Clone, Inspectable, InspectValue, SerializableComponent, Serialize, Deserialize,
)]
pub struct Physics {
    pub mass: f32,
    pub friction: f32,
    pub gravity: f32,
    pub is_gravity_enabled: bool,
    /// Linear damping for physics objects
    pub linear_damping: f32,
    /// Angular damping for physics objects
    pub angular_damping: f32,
    /// Resting threshold - objects below this velocity are considered at rest
    pub resting_threshold: f32,
}

impl Default for Physics {
    fn default() -> Self {
        Self {
            mass: 1.0,
            friction: 0.0,
            gravity: -9.81,
            is_gravity_enabled: false,
            linear_damping: crate::engine::physics::constants::source_physics::PHYSICS_LINEAR_DAMPING,
            angular_damping: crate::engine::physics::constants::source_physics::PHYSICS_ANGULAR_DAMPING,
            resting_threshold: crate::engine::physics::constants::source_physics::RESTING_THRESHOLD,
        }
    }
}

// physics.rs
#[fixed_update]
pub fn apply_velocity_to_transforms(world: &mut World, delta_time: f32) {
    // First collect nodes that have both Transform and Velocity components
    let mut nodes_with_velocity = Vec::new();
    {
        let nodes = world.get_all_nodes();
        for node in nodes {
            // Skip player nodes - they handle their own velocity application
            if node.get_component::<crate::engine::nodes::components::player::Player>().is_some() {
                continue;
            }
            if node.get_component::<Velocity>().is_some() && node.get_component::<Transform>().is_some() {
                nodes_with_velocity.push(node.id);
            }
        }
    }

    // Then apply velocity to those nodes
    for node_id in nodes_with_velocity {
        if let Some(node) = world.get_all_nodes_mut().iter_mut().find(|n| n.id == node_id) {
            let (transform, velocity) = node.get_components_mut::<(&mut Transform, &mut Velocity)>();
            // Apply velocity to position
            transform.position += velocity.direction * delta_time;
        }
    }
}

#[fixed_update]
pub fn apply_physics_damping(world: &mut World, delta_time: f32) {
    // First collect physics data
    let mut physics_data = Vec::new();
    {
        let nodes = world.get_all_nodes();
        for node in nodes {
            if let Some(physics) = node.get_component::<Physics>() {
                physics_data.push((node.id, physics.clone()));
            }
        }
    }

    // Then apply damping
    for (node_id, physics) in physics_data {
        if let Some(node) = world.get_all_nodes_mut().iter_mut().find(|n| n.id == node_id) {
            if let Some(velocity) = node.get_component_mut::<Velocity>() {
                // Apply linear damping
                let linear_damping_factor = (-physics.linear_damping * delta_time).exp();
                velocity.direction *= linear_damping_factor;

                // Apply angular damping
                let angular_damping_factor = (-physics.angular_damping * delta_time).exp();
                velocity.angular_direction *= angular_damping_factor;

                // Check for resting state
                if velocity.direction.magnitude() < physics.resting_threshold
                    && velocity.angular_direction.magnitude() < physics.resting_threshold {
                    velocity.direction = Vector3::zero();
                    velocity.angular_direction = Vector3::zero();
                }
            }
        }
    }
}

#[fixed_update]
pub fn apply_angular_velocity(world: &mut World, delta_time: f32) {
    for node in world.get_all_nodes_mut() {
        if let Some(collider) = node.get_component::<Collider>() {
            if let ColliderShape::Sphere { .. } = collider.shape {
                let (transform, velocity) =
                    node.get_components_mut::<(&mut Transform, &mut Velocity)>();

                let angle = velocity.angular_direction.magnitude();
                if angle > 1e-5 {
                    let axis = velocity.angular_direction / angle;
                    let delta_rotation =
                        Quaternion::from_axis_angle(axis.into(), cgmath::Rad(angle * delta_time));
                    transform.rotation = (delta_rotation * transform.rotation).normalize();
                }
            }
        }
    }
}
