use std::collections::HashSet;

use crate as apostasy;
use apostasy_macros::Resource;
use winit::{
    dpi::PhysicalPosition,
    event::{MouseButton, WindowEvent},
    keyboard::PhysicalKey,
};

#[derive(Resource, Default)]
pub struct InputManager {
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

pub fn is_key_held(input_manager: &InputManager, key: PhysicalKey) -> bool {
    input_manager.keys_held.contains(&key)
}
