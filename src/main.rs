use apostasy::engine::{
    ecs::{
        World,
        components::{
            camera::Camera,
            controllable::Controllable,
            transform::{
                Transform, calculate_forward, calculate_right, calculate_rotation, calculate_up,
            },
            velocity::{Velocity, add_velocity, apply_velocity},
        },
        resources::input_manager::{
            InputManager, KeyAction, KeyBind, is_keybind_active, register_keybind,
        },
    },
    start_app,
};
use apostasy_macros::{Resource, fixed_update, start};
use cgmath::{Deg, Quaternion, Rotation3, Vector3, Zero, num_traits::clamp};
use winit::keyboard::{KeyCode, PhysicalKey};

#[derive(Resource)]
pub struct MyResource {
    pub value: i32,
}

fn main() {
    let world = World::new();

    let rotation = Quaternion::from_axis_angle(Vector3::new(1.0, 0.0, 0.0), Deg(35.0));

    world
        .spawn()
        .insert(Camera::default())
        .insert(Velocity::default())
        .insert(Transform {
            position: Vector3::new(0.0, -2.0, 2.0),
            rotation,
            ..Default::default()
        })
        .insert(Controllable);

    world.insert_resource::<InputManager>(InputManager::default());

    start_app(world).unwrap();
}

#[start]
pub fn kebind_registration(world: &mut World) {
    world.with_resource_mut::<InputManager, _>(|input_manager| {
        register_keybind(
            input_manager,
            KeyBind::new(PhysicalKey::Code(KeyCode::KeyW), KeyAction::Hold),
            "forward",
        );
        register_keybind(
            input_manager,
            KeyBind::new(PhysicalKey::Code(KeyCode::KeyS), KeyAction::Hold),
            "backward",
        );
        register_keybind(
            input_manager,
            KeyBind::new(PhysicalKey::Code(KeyCode::KeyA), KeyAction::Hold),
            "left",
        );
        register_keybind(
            input_manager,
            KeyBind::new(PhysicalKey::Code(KeyCode::KeyD), KeyAction::Hold),
            "right",
        );
        register_keybind(
            input_manager,
            KeyBind::new(PhysicalKey::Code(KeyCode::KeyE), KeyAction::Hold),
            "up",
        );
        register_keybind(
            input_manager,
            KeyBind::new(PhysicalKey::Code(KeyCode::KeyQ), KeyAction::Hold),
            "down",
        );
    });
}

#[fixed_update]
pub fn input_handle(world: &mut World, delta_time: f32) {
    world
        .query()
        .include::<Controllable>()
        .include::<Transform>()
        .include::<Velocity>()
        .build()
        .run_with_resources(|entity, mantle| {
            let resources = mantle.resources.read();
            if let Some(input_manager) = resources.get::<InputManager>() {
                let mut velocity = entity.get_mut::<Velocity>().unwrap();
                let mut transform = entity.get_mut::<Transform>().unwrap();
                if is_keybind_active(input_manager, "forward") {
                    add_velocity(&mut velocity, calculate_forward(&transform) * delta_time);
                }
                if is_keybind_active(input_manager, "backward") {
                    add_velocity(&mut velocity, -calculate_forward(&transform) * delta_time);
                }
                if is_keybind_active(input_manager, "right") {
                    add_velocity(&mut velocity, calculate_right(&transform) * delta_time);
                }
                if is_keybind_active(input_manager, "left") {
                    add_velocity(&mut velocity, -calculate_right(&transform) * delta_time);
                }
                if is_keybind_active(input_manager, "up") {
                    add_velocity(&mut velocity, calculate_up(&transform) * delta_time);
                }
                if is_keybind_active(input_manager, "down") {
                    add_velocity(&mut velocity, -calculate_up(&transform) * delta_time);
                }
                apply_velocity(&velocity, &mut transform);

                transform.yaw += -input_manager.mouse_delta.0 as f32;
                transform.pitch += -input_manager.mouse_delta.1 as f32;

                calculate_rotation(&mut transform);

                velocity.direction = Vector3::zero();
            }
        });
}
