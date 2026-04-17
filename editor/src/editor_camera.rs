use apostasy_core::{
    anyhow::Result,
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

    let direction = inputs.input_vector_3d(
        "Right",
        "Left",
        "Upwards",
        "Downwards",
        "Backwards",
        "Forwards",
    );

    let camera = world.get_object_with_tag_mut::<EditorCamera>().unwrap();
    let velocity = camera.get_component_mut::<Velocity>()?;

    velocity.linear_velocity = direction;

    Ok(())
}
