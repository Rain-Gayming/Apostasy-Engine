use apostasy_core::{
    anyhow::Result,
    cgmath::num_traits::clamp,
    objects::{
        components::transform::Transform, resources::input_manager::InputManager,
        tags::skips_serilization::SkipsSerilization, world::World,
    },
    physics::velocity::Velocity,
    rendering::components::camera::{Camera, EditorCamera},
    start, update,
};

#[start]
pub fn start(world: &mut World) -> Result<()> {
    world
        .add_new_object()
        .add_component(Velocity::default())
        .add_component(Camera::default())
        .add_component(Transform::default())
        .add_tag(EditorCamera)
        .add_tag(SkipsSerilization);

    Ok(())
}

#[update]
pub fn update(world: &mut World) -> Result<()> {
    let inputs = world.get_resource_mut::<InputManager>()?;

    let mouse_delta = inputs.mouse_delta;
    let look_keyboard = inputs.input_vector_2d("LookRight", "LookLeft", "LookUp", "LookDown") * 5.0;
    let direction = inputs.input_vector_3d(
        "Right",
        "Left",
        "Upwards",
        "Downwards",
        "Backwards",
        "Forwards",
    );

    let camera = world.get_object_with_tag_mut::<EditorCamera>()?;
    let rotation = {
        let transform = camera.get_component::<Transform>()?;

        transform.global_rotation
    };

    let velocity = camera.get_component_mut::<Velocity>()?;

    velocity.linear_velocity = rotation * direction;

    let transform = camera.get_component_mut::<Transform>()?;
    println!("{:?}", transform);
    transform.local_euler_angles.y -= mouse_delta.0 as f32;
    transform.local_euler_angles.x = clamp(
        transform.local_euler_angles.x - mouse_delta.1 as f32,
        -90.0,
        90.0,
    );

    transform.local_euler_angles.y -= look_keyboard.x as f32;
    transform.local_euler_angles.x = clamp(
        transform.local_euler_angles.x - look_keyboard.y as f32,
        -90.0,
        90.0,
    );

    Ok(())
}
