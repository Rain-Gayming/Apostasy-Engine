use std::collections::HashSet;

use crate::{self as apostasy, engine::ecs::World};
use apostasy_macros::{Resource, late_update};
use egui::ahash::HashMap;
use winit::{
    dpi::PhysicalPosition,
    event::{MouseButton, WindowEvent},
    keyboard::PhysicalKey,
};

pub enum KeyAction {
    Press,
    Release,
    Hold,
}
pub struct KeyBind {
    pub key: PhysicalKey,
    pub action: KeyAction,
}

impl KeyBind {
    pub fn new(key: PhysicalKey, action: KeyAction) -> Self {
        Self { key, action }
    }
}

#[derive(Resource, Default)]
pub struct InputManager {
    keybinds: HashMap<String, KeyBind>,
    keys_held: HashSet<PhysicalKey>,
    mouse_held: HashSet<MouseButton>,
    mouse_position: PhysicalPosition<f64>,
    mouse_delta: (f64, f64),
    // scroll_delta: (f32, f32),

    // Resets each frame
    keys_pressed: HashSet<PhysicalKey>,
    keys_released: HashSet<PhysicalKey>,
    mouse_pressed: HashSet<MouseButton>,
    mouse_released: HashSet<MouseButton>,
}

pub fn register_keybind(input_manager: &mut InputManager, key: KeyBind, name: &str) {
    input_manager.keybinds.insert(name.to_string(), key);
}

pub fn is_keybind_active(input_manager: &InputManager, name: &str) -> bool {
    let key = input_manager.keybinds.get(name).unwrap();
    match key.action {
        KeyAction::Press => input_manager.keys_pressed.contains(&key.key),
        KeyAction::Release => input_manager.keys_released.contains(&key.key),
        KeyAction::Hold => input_manager.keys_held.contains(&key.key),
    }
}

#[late_update]
pub fn clear_actions(world: &mut World) {
    world.with_resource_mut::<InputManager, _>(|input_manager| {
        input_manager.keys_pressed.clear();
        input_manager.keys_released.clear();
        input_manager.mouse_pressed.clear();
        input_manager.mouse_released.clear();
    });
}

pub fn handle_input_event(input_manager: &mut InputManager, event: WindowEvent) {
    match event {
        WindowEvent::KeyboardInput { event, .. } => {
            if event.state.is_pressed() {
                input_manager.keys_pressed.insert(event.physical_key);
                input_manager.keys_held.insert(event.physical_key);
            } else {
                input_manager.keys_released.insert(event.physical_key);
                input_manager.keys_held.remove(&event.physical_key);
            }
        }
        WindowEvent::MouseInput { state, button, .. } => {
            if state.is_pressed() {
                input_manager.mouse_pressed.insert(button);
                input_manager.mouse_held.insert(button);
            } else {
                input_manager.mouse_released.insert(button);
                input_manager.mouse_held.remove(&button);
            }
        }
        WindowEvent::CursorMoved { position, .. } => {
            let delta = (
                position.x - input_manager.mouse_position.x,
                position.y - input_manager.mouse_position.y,
            );
            input_manager.mouse_delta = delta;
            input_manager.mouse_position = position;
        }
        _ => {}
    }
}
