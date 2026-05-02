use std::default;

use apostasy_core::{
    anyhow::Result,
    cgmath::Vector3,
    fixed_update,
    objects::{
        Object,
        components::transform::Transform,
        resources::{
            cursor_manager::CursorManager, input_manager::InputManager,
            window_manager::WindowManager,
        },
        systems::DeltaTime,
        tags::Player,
        world::World,
    },
    physics::{Gravity, collider::Collider, velocity::Velocity},
    rendering::components::camera::{ActiveCamera, Camera, GameCamera},
    start, update,
    voxels::voxel_raycast::voxel_raycast_system,
};

use crate::entities::spawn_point::NeedsSpawnPoint;

#[start]
pub fn player_init(world: &mut World) -> Result<()> {
    let transform = Transform {
        local_position: Vector3::new(0.0, 50.0, 0.0),
        ..Default::default()
    };

    let camera = Object::new()
        .add_component(Transform {
            local_position: Vector3 {
                x: 0.0,
                y: 0.8,
                z: 0.0,
            },
            ..Default::default()
        })
        .add_component(Camera::default())
        .add_tag(ActiveCamera)
        .add_tag(GameCamera);
    let player = Object::new()
        .add_component(transform)
        .add_component(Velocity::default())
        .add_component(Gravity::default())
        .add_component(Collider::player())
        .add_tag(Player)
        .add_tag(NeedsSpawnPoint);

    let player_id = world.add_object(player.clone());
    let cam_id = world.add_object(camera.clone());
    world.set_parent(cam_id, Some(player_id))?;
    Ok(())
}

#[start]
pub fn player_start(world: &mut World) -> Result<()> {
    {
        let cursor_manager = world.get_resource_mut::<CursorManager>()?;

        cursor_manager.set_mode(
            apostasy_core::objects::resources::cursor_manager::CursorLockMode::LockedHidden,
        );
    }

    {
        let cursor_manager = world.get_resource::<CursorManager>()?.clone();
        let window_manager = world.get_resource_mut::<WindowManager>()?;
        cursor_manager.update_cursor(window_manager);
    }

    Ok(())
}

#[update]
pub fn update(world: &mut World) -> Result<()> {
    let inputs = world.get_resource::<InputManager>()?;
    let delta = world.get_resource::<DeltaTime>()?.0;

    let mouse_delta = inputs.mouse_delta;
    let direction = inputs.input_vector_2d("Right", "Left", "Backwards", "Forwards");
    let should_jump = inputs.is_keybind_active("Jump");

    let player = world.get_object_with_tag_mut::<Player>()?;
    let player_transform = player.get_component_mut::<Transform>()?;
    player_transform.local_euler_angles.y -= mouse_delta.0 as f32 * delta * 50.0;

    let camera = world.get_object_with_tag_mut::<GameCamera>()?;
    let cam_transform = camera.get_component_mut::<Transform>()?;
    cam_transform.local_euler_angles.x -= mouse_delta.1 as f32 * delta * 50.0;
    cam_transform.local_euler_angles.x = cam_transform.local_euler_angles.x.clamp(-89.0, 89.0);

    let player = world.get_object_with_tag::<Player>()?;
    let rotation = player.get_component::<Transform>()?.global_rotation;

    let player = world.get_object_with_tag_mut::<Player>()?;
    let velocity = player.get_component_mut::<Velocity>()?;

    let wish_dir = rotation * Vector3::new(direction.x, 0.0, direction.y);
    velocity.linear_velocity.x = wish_dir.x * 2.0;
    velocity.linear_velocity.z = wish_dir.z * 2.0;

    if should_jump && velocity.is_grounded {
        velocity.linear_velocity.y = 8.0;
    }

    Ok(())
}

#[fixed_update]
pub fn block_updates(world: &mut World, _elta: f32) -> Result<()> {
    let inputs = world.get_resource::<InputManager>()?;
    let to_break = inputs.is_mousebind_active("Break");
    let to_place = inputs.is_mousebind_active("Place");

    if to_break {
        voxel_raycast_system(world, Some(0))?;
    }
    if to_place {
        voxel_raycast_system(world, Some(2))?;
    }

    Ok(())
}
