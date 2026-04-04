use crate::engine::editor::inspectable::Inspectable;
use crate::{self as apostasy, engine::nodes::components::transform::Transform};
use apostasy_macros::{Component, InspectValue, Inspectable, SerializableComponent};
use cgmath::{InnerSpace, Vector3, Zero};
use serde::{Deserialize, Serialize};

#[derive(
    Component, Clone, Inspectable, InspectValue, SerializableComponent, Serialize, Deserialize,
)]
pub struct Velocity {
    pub direction: Vector3<f32>,
    pub angular_direction: Vector3<f32>,
    pub time_since_ground: f32,
    pub is_grounded: bool,
}

impl Default for Velocity {
    fn default() -> Self {
        Self {
            direction: Vector3::new(0.0, 0.0, 0.0),
            angular_direction: Vector3::new(0.0, 0.0, 0.0),
            time_since_ground: 0.0,
            is_grounded: false,
        }
    }
}

impl Velocity {
    /// Apply acceleration-based movement
    pub fn accelerate(
        &mut self,
        wish_dir: Vector3<f32>,
        wish_speed: f32,
        acceleration: f32,
        delta_time: f32,
    ) {
        let current_speed = self.direction.dot(wish_dir);
        let add_speed = wish_speed - current_speed;

        if add_speed <= 0.0 {
            return;
        }

        let mut accel_speed = acceleration * wish_speed * delta_time;

        if accel_speed > add_speed {
            accel_speed = add_speed;
        }

        self.direction += wish_dir * accel_speed;
    }

    /// Apply ground friction
    pub fn apply_ground_friction(&mut self, friction: f32, _stop_speed: f32, delta_time: f32) {
        let speed = self.direction.magnitude();

        if speed < 0.1 {
            self.direction = Vector3::zero();
            return;
        }

        let speed_drop = speed * friction * delta_time;
        let new_speed = (speed - speed_drop).max(0.0);

        if new_speed > 0.0 {
            self.direction *= new_speed / speed;
        } else {
            self.direction = Vector3::zero();
        }
    }

    /// Apply air friction
    pub fn apply_air_friction(&mut self, delta_time: f32) {
        // Air friction is very low in Source
        let air_friction: f32 = 0.99;
        self.direction *= air_friction.powf(delta_time);
    }

    /// Check if we can bunny hop
    pub fn can_bunny_hop(&self, grace_period: f32) -> bool {
        self.time_since_ground < grace_period
    }

    /// Jump with momentum preservation
    pub fn jump(&mut self, jump_speed: f32, bunny_hop_factor: f32) {
        if self.can_bunny_hop(crate::engine::physics::constants::source_physics::JUMP_GRACE_PERIOD)
        {
            // Preserve horizontal speeds
            let horizontal_vel = Vector3::new(self.direction.x, 0.0, self.direction.z);
            let preserved_speed = horizontal_vel.magnitude() * bunny_hop_factor;
            let preserved_dir = if horizontal_vel.magnitude() > 0.1 {
                horizontal_vel.normalize()
            } else {
                Vector3::new(0.0, 0.0, 1.0)
            };

            self.direction.x = preserved_dir.x * preserved_speed;
            self.direction.z = preserved_dir.z * preserved_speed;
        } else {
            // Normal jump reset horizontal velocity
            self.direction.x = 0.0;
            self.direction.z = 0.0;
        }

        self.direction.y = jump_speed;
        self.time_since_ground = 0.0;
        self.is_grounded = false;
    }

    /// Update ground state
    pub fn update_ground_state(&mut self, is_grounded: bool, delta_time: f32) {
        self.is_grounded = is_grounded;
        if is_grounded {
            self.time_since_ground = 0.0;
        } else {
            self.time_since_ground += delta_time;
        }
    }

    /// Clamp horizontal speed
    pub fn clamp_horizontal_speed(&mut self, max_speed: f32) {
        let horizontal_vel = Vector3::new(self.direction.x, 0.0, self.direction.z);
        let speed = horizontal_vel.magnitude();

        if speed > max_speed {
            let clamped_vel = horizontal_vel.normalize() * max_speed;
            self.direction.x = clamped_vel.x;
            self.direction.z = clamped_vel.z;
        }
    }

    /// Add velocity (for external forces)
    pub fn add_velocity(&mut self, velocity: Vector3<f32>) {
        self.direction += velocity;
    }

    /// Synchronize linear velocity from angular velocity
    pub fn sync_linear_from_angular(&mut self, radius: f32, normal: Vector3<f32>) {
        let tangential_vel = self.angular_direction.cross(normal) * radius;
        let normal_component = self.direction.dot(normal) * normal;
        self.direction = normal_component + tangential_vel;
    }
}

pub fn apply_velocity(velocity: &mut Velocity, transform: &mut Transform, delta_time: f32) {
    transform.position += velocity.direction * delta_time;
}
