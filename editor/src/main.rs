use apostasy_core::{
    anyhow::Result,
    init_core, log,
    objects::{
        resources::input_manager::{InputManager, KeyAction, KeyBind},
        world::World,
    },
    physics::velocity::Velocity,
    rendering::{
        RenderingBackend,
        components::camera::{Camera, MainCamera},
    },
    start, update,
    winit::keyboard::{KeyCode, PhysicalKey},
};

fn main() {
    init_core(RenderingBackend::Vulkan).unwrap();
}

#[start]
pub fn start(world: &mut World) -> Result<()> {
    let inputs = world.get_resource_mut::<InputManager>()?;

    inputs.register_keybind(KeyBind::new(
        PhysicalKey::Code(KeyCode::KeyA),
        KeyAction::Press,
        "A",
    ));

    Ok(())
}

#[update]
pub fn update(world: &mut World) -> Result<()> {
    let inputs = world.get_resource_mut::<InputManager>()?;

    if inputs.is_keybind_active("A") {
        log!("A");
        let camera = world.get_object_with_tag_mut::<MainCamera>().unwrap();
        let velocity = camera.get_component_mut::<Velocity>()?;
        velocity.linear_velocity.x += 0.001;
    }

    Ok(())
}
