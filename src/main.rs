use apostasy::engine::{
    nodes::{
        World,
        components::{
            physics::Physics,
            player::Player,
            raycast::Raycast,
            transform::Transform,
            velocity::{Velocity, apply_velocity},
        },
    },
    start_app,
};
use apostasy_macros::fixed_update;
use cgmath::{Vector3, Zero, num_traits::clamp};

fn main() {
    start_app().unwrap();
    println!("Hello, world!");
}

#[fixed_update]
pub fn player_movement(world: &mut World, delta_time: f32) {
    let mouse_delta = world.input_manager.mouse_delta;
    let input_dir = world
        .input_manager
        .input_vector_3d("right", "left", "up", "down", "backward", "forward");

    let children = world.get_all_nodes();
    let nodes = world.get_all_nodes();

    let mut is_grounded: bool = false;
    for child in children {
        if let Some(rc) = child.get_component::<Raycast>()
            && let Some(transform) = child.get_component::<Transform>()
            && let Some(_) = rc.cast(transform, &nodes, "")
        {
            is_grounded = true;
        } else {
            is_grounded = false;
        }
    }

    let player = world.get_node_with_component_mut::<Player>();

    if let Some(player) = player {
        let (camera_transform, velocity, physics) =
            player.get_components_mut::<(&mut Transform, &mut Velocity, &mut Physics)>();
        camera_transform.rotation_euler.y -= mouse_delta.0 as f32;
        camera_transform.rotation_euler.x = clamp(
            camera_transform.rotation_euler.x - mouse_delta.1 as f32,
            -89.0,
            89.0,
        );

        camera_transform.calculate_rotation();

        let direction = camera_transform.global_rotation * input_dir;

        physics.is_gravity_enabled = !is_grounded;

        velocity.add_velocity(direction * delta_time);

        velocity.direction *= delta_time;
        apply_velocity(velocity, camera_transform);
        velocity.direction = Vector3::zero();
    }
}
