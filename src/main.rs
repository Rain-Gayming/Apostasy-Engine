use apostasy::engine::{
    ecs::{
        World,
        components::{
            camera::Camera,
            controllable::Controllable,
            transform::{Transform, calculate_rotation},
            velocity::{Velocity, add_velocity, apply_velocity},
        },
        entity::EntityView,
        resource,
        resources::input_manager::{
            InputManager, KeyAction, KeyBind, input_vector_3d, is_keybind_active, register_keybind,
        },
    },
    start_app,
    windowing::{CursorManager, WindowManager, grab_cursor, toggle_hide_cursor},
};
use apostasy_macros::{Resource, fixed_update, start};
use cgmath::{Deg, Quaternion, Rotation3, Vector3, Zero};
use winit::keyboard::{KeyCode, PhysicalKey};

#[derive(Resource)]
pub struct MyResource {
    pub value: i32,
}

fn main() {
    let world = World::new();

    start_app(world).unwrap();
}

#[start(priority = 1)]
pub fn start(world: &mut World) {
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
    world.insert_resource::<CursorManager>(CursorManager::default());
}

#[start]
pub fn keybind_registration(world: &mut World) {
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
        register_keybind(
            input_manager,
            KeyBind::new(PhysicalKey::Code(KeyCode::Escape), KeyAction::Press),
            "pause",
        );
    });
}

#[fixed_update]
pub fn input_handle(world: &mut World, delta_time: f32) {
    let mut pauisng = false;
    world
        .query()
        .include::<Controllable>()
        .include::<Transform>()
        .include::<Velocity>()
        .build()
        .run_with_resources(|entity, mantle| {
            world.with_resource_mut::<InputManager, _>(|input_manager| {
                let mut velocity = entity.get_mut::<Velocity>().unwrap();
                let mut transform = entity.get_mut::<Transform>().unwrap();

                let direction = transform.rotation
                    * input_vector_3d(
                        input_manager,
                        "right",
                        "left",
                        "up",
                        "down",
                        "backward",
                        "forward",
                    );
                add_velocity(&mut velocity, direction * delta_time);

                apply_velocity(&velocity, &mut transform);

                transform.yaw += -input_manager.mouse_delta.0 as f32;
                transform.pitch += input_manager.mouse_delta.1 as f32;

                calculate_rotation(&mut transform);

                velocity.direction = Vector3::zero();

                if is_keybind_active(input_manager, "pause") {
                    pauisng = true;
                }
            });

            if pauisng {
                world.with_resources::<(WindowManager, CursorManager), _, _>(
                    |(window_manager, cursor_manager)| {
                        grab_cursor(cursor_manager, window_manager);
                    },
                );
            }
        });
}
