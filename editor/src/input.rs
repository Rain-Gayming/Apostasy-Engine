use apostasy_core::{
    anyhow::Result,
    objects::{
        resources::input_manager::{InputManager, KeyAction, KeyBind},
        world::World,
    },
    start,
    winit::keyboard::{KeyCode, PhysicalKey},
};

#[start]
pub fn start(world: &mut World) -> Result<()> {
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

    Ok(())
}
