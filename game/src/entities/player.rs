use apostasy_core::{
    anyhow::Result,
    cgmath::{Vector3, num_traits::clamp},
    log,
    objects::{
        Object,
        components::transform::Transform,
        resources::{
            cursor_manager::CursorManager,
            input_manager::{InputManager, KeyAction, KeyBind, MouseBind},
            window_manager::WindowManager,
        },
        systems::DeltaTime,
        tags::Player,
        world::World,
    },
    physics::velocity::Velocity,
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
        .add_component(transform.clone())
        .add_component(Camera::default())
        .add_tag(ActiveCamera)
        .add_tag(GameCamera);
    let player = Object::new()
        .add_component(transform)
        .add_component(Velocity::default())
        .add_tag(Player)
        .add_tag(NeedsSpawnPoint);

    world.set_parent(camera.id, Some(player.id))?;

    world.add_object(player.clone());
    Ok(())
}

#[start]
pub fn player_start(world: &mut World) -> Result<()> {
    {
        let cursor_manager = world.get_resource_mut::<CursorManager>()?;

        cursor_manager.switch_mode();
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
        PhysicalKey::Code(KeyCode::KeyE),
        KeyAction::Hold,
        "Upwards",
    ));
    inputs.register_keybind(KeyBind::new(
        PhysicalKey::Code(KeyCode::KeyQ),
        KeyAction::Hold,
        "Downwards",
    ));

    inputs.register_keybind(KeyBind::new(
        PhysicalKey::Code(KeyCode::ArrowLeft),
        KeyAction::Hold,
        "LookLeft",
    ));
    inputs.register_keybind(KeyBind::new(
        PhysicalKey::Code(KeyCode::ArrowRight),
        KeyAction::Hold,
        "LookRight",
    ));
    inputs.register_keybind(KeyBind::new(
        PhysicalKey::Code(KeyCode::ArrowUp),
        KeyAction::Hold,
        "LookUp",
    ));
    inputs.register_keybind(KeyBind::new(
        PhysicalKey::Code(KeyCode::ArrowDown),
        KeyAction::Hold,
        "LookDown",
    ));

    inputs.register_mousebind(MouseBind::new(MouseButton::Left, KeyAction::Hold, "Break"));
    inputs.register_mousebind(MouseBind::new(MouseButton::Right, KeyAction::Hold, "Place"));

    Ok(())
}

#[update]
pub fn update(world: &mut World) -> Result<()> {
    let delta = world.get_resource::<DeltaTime>()?.0;
    let inputs = world.get_resource_mut::<InputManager>()?;

    let mouse_delta = inputs.mouse_delta;
    let look_keyboard = inputs.input_vector_2d("LookRight", "LookLeft", "LookUp", "LookDown") * 5.0;
    let to_break = inputs.is_mousebind_active("Break");
    let to_place = inputs.is_mousebind_active("Place");
    let direction = inputs.input_vector_3d(
        "Right",
        "Left",
        "Upwards",
        "Downwards",
        "Backwards",
        "Forwards",
    );

    let camera = world.get_object_with_tag_mut::<GameCamera>()?;
    let rotation = {
        let transform = camera.get_component::<Transform>()?;

        transform.global_rotation
    };

    let velocity = camera.get_component_mut::<Velocity>()?;

    velocity.linear_velocity = rotation * direction * delta * 5.0;

    let transform = camera.get_component_mut::<Transform>()?;
    transform.local_euler_angles.y -= mouse_delta.0 as f32 * 4.0;
    transform.local_euler_angles.x = clamp(
        transform.local_euler_angles.x - mouse_delta.1 as f32 * 4.0,
        -90.0,
        90.0,
    );

    transform.local_euler_angles.y -= look_keyboard.x as f32;
    transform.local_euler_angles.x = clamp(
        transform.local_euler_angles.x - look_keyboard.y as f32,
        -90.0,
        90.0,
    );

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
    // Only run if player still needs a spawn point
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
            hit.voxel_pos.y as f32 + 3.0,
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
