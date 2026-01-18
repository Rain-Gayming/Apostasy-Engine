use apostasy::engine::{
    ecs::{
        World,
        components::{
            camera::Camera,
            controllable::Controllable,
            transform::{Transform, calculate_forward, calculate_right},
            velocity::{Velocity, add_velocity, apply_velocity},
        },
        resources::input_manager::{InputManager, is_key_held},
    },
    start_app,
};
use apostasy_macros::{Resource, update};
use cgmath::{Deg, Quaternion, Rotation3, Vector3, Zero};

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

#[update]
pub fn input_handle(world: &mut World) {
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
                if is_key_held(input_manager, PhysicalKey::Code(KeyCode::KeyW)) {
                    add_velocity(&mut velocity, calculate_forward(&transform));
                }
                if is_key_held(input_manager, PhysicalKey::Code(KeyCode::KeyS)) {
                    add_velocity(&mut velocity, -calculate_forward(&transform));
                }
                if is_key_held(input_manager, PhysicalKey::Code(KeyCode::KeyD)) {
                    add_velocity(&mut velocity, calculate_right(&transform));
                }
                if is_key_held(input_manager, PhysicalKey::Code(KeyCode::KeyA)) {
                    add_velocity(&mut velocity, -calculate_right(&transform));
                }
                apply_velocity(&velocity, &mut transform);
                velocity.direction = Vector3::zero();
            }
        });
}
