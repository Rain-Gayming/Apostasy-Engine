use apostasy_core::{
    anyhow::Result,
    cgmath::Vector3,
    log,
    objects::{
        Object,
        components::transform::Transform,
        resources::{
            cursor_manager::CursorManager,
            input_manager::{InputManager, KeyAction, KeyBind, MouseBind},
            window_manager::WindowManager,
        },
        tags::Player,
        world::World,
    },
    physics::{Gravity, collider::Collider, velocity::Velocity},
    rendering::components::camera::{ActiveCamera, Camera, GameCamera},
    start, update,
    voxels::voxel_raycast::{Direction, voxel_raycast, voxel_raycast_system},
    winit::{
        event::MouseButton,
        keyboard::{KeyCode, PhysicalKey},
    },
};
use apostasy_macros::Tag;

#[start]
pub fn player_init(world: &mut World) -> Result<()> {
    let transform = Transform::default();

    let camera = Object::new()
        .add_component(Transform {
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

#[start]
pub fn player_inputs(world: &mut World) -> Result<()> {
    let inputs = world.get_resource_mut::<InputManager>()?;

    inputs.register_keybind(KeyBind::new(
        PhysicalKey::Code(KeyCode::KeyA),
        KeyAction::Hold,
        "Left",
    ));
    inputs.register_keybind(KeyBind::new(
        PhysicalKey::Code(KeyCode::KeyD),
        KeyAction::Hold,
        "Right",
    ));
    inputs.register_keybind(KeyBind::new(
        PhysicalKey::Code(KeyCode::KeyW),
        KeyAction::Hold,
        "Forwards",
    ));
    inputs.register_keybind(KeyBind::new(
        PhysicalKey::Code(KeyCode::KeyS),
        KeyAction::Hold,
        "Backwards",
    ));
    inputs.register_keybind(KeyBind::new(
        PhysicalKey::Code(KeyCode::Space),
        KeyAction::Press,
        "Jump",
    ));
    inputs.register_keybind(KeyBind::new(
        PhysicalKey::Code(KeyCode::KeyQ),
        KeyAction::Hold,
        "Downwards",
    ));

    inputs.register_mousebind(MouseBind::new(MouseButton::Left, KeyAction::Hold, "Break"));
    inputs.register_mousebind(MouseBind::new(MouseButton::Right, KeyAction::Hold, "Place"));

    Ok(())
}

#[update]
pub fn update(world: &mut World) -> Result<()> {
    let inputs = world.get_resource::<InputManager>()?;

    let mouse_delta = inputs.mouse_delta;
    let to_break = inputs.is_mousebind_active("Break");
    let to_place = inputs.is_mousebind_active("Place");
    let direction = inputs.input_vector_2d("Right", "Left", "Backwards", "Forwards");
    let should_jump = inputs.is_keybind_active("Jump");

    let camera = world.get_object_with_tag::<GameCamera>()?;
    let rotation = {
        let transform = camera.get_component::<Transform>()?;
        transform.global_rotation
    };

    let player = world.get_object_with_tag_mut::<Player>()?;

    let player_transform = player.get_component_mut::<Transform>()?;
    player_transform.local_euler_angles.y -= mouse_delta.0 as f32 * 4.0;

    let velocity = player.get_component_mut::<Velocity>()?;

    let wish_dir = rotation * Vector3::new(direction.x, 0.0, direction.y);

    velocity.linear_velocity.x = wish_dir.x * 5.0;
    velocity.linear_velocity.z = wish_dir.z * 5.0;

    if should_jump && velocity.is_grounded {
        velocity.linear_velocity.y = 8.0;
        velocity.is_grounded = false;
    }

    if to_break {
        voxel_raycast_system(world, Some(0))?;
    }
    if to_place {
        voxel_raycast_system(world, Some(2))?;
    }

    Ok(())
}

#[derive(Tag, Clone)]
pub struct NeedsSpawnPoint;

#[update]
pub fn find_spawn_point(world: &mut World) -> Result<()> {
    let player = world.get_object_with_tag::<Player>()?;
    if player.get_tag::<NeedsSpawnPoint>().is_err() {
        return Ok(());
    }

    let transform = Transform {
        local_position: Vector3::new(0.0, 256.0, 0.0),
        global_position: Vector3::new(0.0, 256.0, 0.0),
        ..Default::default()
    };

    if let Ok(hit) = voxel_raycast(world, &transform, 1000, Direction::Down) {
        let spawn = Vector3::new(
            hit.voxel_pos.x as f32,
            hit.voxel_pos.y as f32 + 5.0,
            hit.voxel_pos.z as f32,
        );
        log!("Found spawn point at {:?}", spawn);

        let player = world.get_object_with_tag_mut::<Player>()?;
        let t = player.get_component_mut::<Transform>()?;
        t.local_position = spawn;
        t.global_position = spawn;
        player.remove_tag::<NeedsSpawnPoint>();
    }

    Ok(())
}
