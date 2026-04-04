pub mod custom_components;

use apostasy::engine::{
    nodes::{
        components::{camera::Camera, player::Player, transform::Transform},
        world::World,
    },
    physics::{
        constants::source_physics,
        physics_body::PhysicsBody,
        velocity::{self, Velocity},
    },
    start_app,
    windowing::cursor_manager::{CursorLockMode, CursorManager},
};
use apostasy_macros::{fixed_update, start, update};
use cgmath::{InnerSpace, Vector3, num_traits::clamp};

use crate::custom_components::movement_stats::MovementStats;

fn main() {
    start_app().unwrap();
    println!("Hello, world!");
}

#[start]
pub fn start(world: &mut World) {
    let cursor_manager = world.get_node_with_component_mut::<CursorManager>();
    if let Some(mut cursor_manager) = cursor_manager {
        cursor_manager
            .get_component_mut::<CursorManager>()
            .unwrap()
            .cursor_lock_mode = CursorLockMode::GrabbedHidden;
    }
}

#[update]
pub fn player_input(world: &mut World) {
    let mouse_delta = world.input_manager.mouse_delta;
    let input_dir = world
        .input_manager
        .input_vector_3d("right", "left", "up", "down", "backward", "forward");

    let player = world.get_node_with_component_mut::<Player>();
    let camera = world.get_node_with_component_mut::<Camera>();

    if let Some(mut player) = player {
        // Handle camera rotation from mouse input
        if let Some(mut camera) = camera {
            let camera_transform = camera.get_component_mut::<Transform>().unwrap();
            camera_transform.rotation_euler.x = clamp(
                camera_transform.rotation_euler.x - mouse_delta.1 as f32,
                -89.0,
                89.0,
            );
            camera_transform.rotation_euler.y -= mouse_delta.0 as f32;
        }

        let player_transform = player.get_component::<Transform>().unwrap();
        let mut calculated_transform = player_transform.clone();
        calculated_transform.calculate_rotation();

        let mut wish_dir: Vector3<f32>;
        if let Some(camera) = world.get_node_with_component::<Camera>() {
            let camera_transform = camera.get_component::<Transform>().unwrap();
            wish_dir = camera_transform.global_rotation * input_dir;
        } else {
            wish_dir = calculated_transform.global_rotation * input_dir;
        }
        wish_dir.y = 0.0;

        if wish_dir.magnitude() > 0.1 {
            wish_dir = wish_dir.normalize();
        }

        let player_component = player.get_component_mut::<Player>().unwrap();
        player_component.wish_dir = wish_dir;

        let jump_key_pressed = world.input_manager.is_keybind_active("jump");
        let jump_pressed_this_frame = jump_key_pressed && !player_component.previous_jump_pressed;
        player_component.previous_jump_pressed = jump_key_pressed;

        if jump_pressed_this_frame {
            println!("Jump");
            let mut jump_factor = source_physics::JUMP_SPEED;

            if let Some(movement_stats) = player.get_component::<MovementStats>() {
                jump_factor = movement_stats.jump_speed;
            }

            if let Some(velocity) = player.get_component_mut::<Velocity>() {
                // Allow jumping if grounded or within coyote time
                if velocity.is_grounded || velocity.time_since_ground < source_physics::COYOTE_TIME
                {
                    velocity.jump(jump_factor, source_physics::BUNNY_HOP_FACTOR);
                } else {
                    println!("Not grounded");
                }
            }
        }
    }
}

#[fixed_update]
pub fn player_movement(world: &mut World, delta: f32) {
    use apostasy::engine::physics::constants::source_physics;

    let player = world.get_node_with_component_mut::<Player>();
    if let Some(player) = player {
        let (_player_transform, velocity, _physics, player_component) = player
            .get_components_mut::<(&mut Transform, &mut Velocity, &mut PhysicsBody, &mut Player)>();

        // Determine acceleration based on ground state
        let acceleration = if velocity.is_grounded {
            source_physics::GROUND_ACCELERATION
        } else {
            source_physics::AIR_ACCELERATION
        };

        // Apply acceleration
        let wish_speed = if let Some(movement_stats) = player.get_component::<MovementStats>() {
            movement_stats.current_speed
        } else {
            source_physics::MAX_GROUND_SPEED
        };

        velocity.accelerate(player_component.wish_dir, wish_speed, acceleration, delta);

        // Clamp horizontal speed
        velocity.clamp_horizontal_speed(source_physics::MAX_GROUND_SPEED);
    }
}

#[fixed_update]
pub fn physics_update(world: &mut World, delta: f32) {
    let player = world.get_node_with_component_mut::<Player>();
    if let Some(player) = player {
        let (player_transform, velocity, _physics) =
            player.get_components_mut::<(&mut Transform, &mut Velocity, &mut PhysicsBody)>();

        velocity::apply_velocity(velocity, player_transform, delta);
    }
}
