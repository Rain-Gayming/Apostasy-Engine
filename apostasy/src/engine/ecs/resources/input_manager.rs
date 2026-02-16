use std::collections::HashSet;

use crate::{self as apostasy, engine::ecs::World};
use apostasy_macros::{Resource, late_update};
use cgmath::{Vector2, Vector3};
use egui::ahash::HashMap;
use winit::{
    dpi::PhysicalPosition,
    event::{DeviceEvent, MouseButton, WindowEvent},
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
    pub mouse_delta: (f64, f64),
    // scroll_delta: (f32, f32),

    // Resets each frame
    keys_pressed: HashSet<PhysicalKey>,
    keys_released: HashSet<PhysicalKey>,
    mouse_pressed: HashSet<MouseButton>,
    mouse_released: HashSet<MouseButton>,
}

/// Rebinds a key, use:
/// ```rust
///     world.with_resource_mut(|input_manager: &mut InputManager| {
///         rebind_key(input_manager, KeyBind::new(PhysicalKey::Code(KeyCode::KeyW), KeyAction::Hold), "forward");
///     });
/// ```
pub fn rebind_key(input_manager: &mut InputManager, key: KeyBind, name: &str) {
    input_manager.keybinds.remove(name);
    input_manager.keybinds.insert(name.to_string(), key);
}

/// Registers a key, use:
/// ```rust
///     world.with_resource_mut(|input_manager: &mut InputManager| {
///         register_key(input_manager, KeyBind::new(PhysicalKey::Code(KeyCode::KeyW), KeyAction::Hold), "forward");
///     });
/// ```
pub fn register_keybind(input_manager: &mut InputManager, key: KeyBind, name: &str) {
    println!("registering keybind: {}", name);
    input_manager.keybinds.insert(name.to_string(), key);
}

/// Checks if a key is active, use:
/// ```rust
///     world.with_resource_mut(|input_manager: &mut InputManager| {
///         if is_keybind_active(input_manager, "forward") {
///             println!("forward is active");
///         }
///     });
/// ```
pub fn is_keybind_active(input_manager: &InputManager, name: &str) -> bool {
    let key = input_manager.keybinds.get(name);
    if key.is_none() {
        return false;
    }
    let key = key.unwrap();
    match key.action {
        KeyAction::Press => input_manager.keys_pressed.contains(&key.key),
        KeyAction::Release => input_manager.keys_released.contains(&key.key),
        KeyAction::Hold => input_manager.keys_held.contains(&key.key),
    }
}

#[late_update]
pub fn clear_actions(world: &mut World) {
    world.with_resource_mut(|input_manager: &mut InputManager| {
        input_manager.keys_pressed.clear();
        input_manager.keys_released.clear();
        input_manager.mouse_pressed.clear();
        input_manager.mouse_released.clear();
        input_manager.mouse_delta = (0.0, 0.0);
    });
}

/// Calculates the input vector for 2D movement, use:
/// ```rust
///     world.with_resource_mut(|input_manager: &mut InputManager| {
///         let direction = input_vector_2d(
///             input_manager,
///             "left",
///             "right",
///             "up",
///             "down",
///         );
///     });
/// ```
pub fn input_vector_2d(
    input_manager: &InputManager,
    left: &str,
    right: &str,
    up: &str,
    down: &str,
) -> Vector2<f32> {
    let mut x = 0.0;
    let mut y = 0.0;
    if is_keybind_active(input_manager, left) {
        x += 1.0;
    }
    if is_keybind_active(input_manager, right) {
        x -= 1.0;
    }
    if is_keybind_active(input_manager, up) {
        y += 1.0;
    }
    if is_keybind_active(input_manager, down) {
        y -= 1.0;
    }
    Vector2::new(x, y)
}

/// Calculates the input vector for 3D movement, use:
/// ```rust
///     world.with_resource_mut(|input_manager: &mut InputManager| {
///         let direction = input_vector_3d(
///             input_manager,
///             "right",
///             "left",
///             "up",
///             "down",
///             "backward",
///             "forward",
///         );
///     });
/// ```
pub fn input_vector_3d(
    input_manager: &InputManager,
    x_pos: &str,
    x_neg: &str,
    y_pos: &str,
    y_neg: &str,
    z_pos: &str,
    z_neg: &str,
) -> Vector3<f32> {
    let mut x = 0.0;
    let mut y = 0.0;
    let mut z = 0.0;
    if is_keybind_active(input_manager, x_pos) {
        x += 1.0;
    }
    if is_keybind_active(input_manager, x_neg) {
        x -= 1.0;
    }
    if is_keybind_active(input_manager, y_pos) {
        y += 1.0;
    }
    if is_keybind_active(input_manager, y_neg) {
        y -= 1.0;
    }
    if is_keybind_active(input_manager, z_pos) {
        z += 1.0;
    }
    if is_keybind_active(input_manager, z_neg) {
        z -= 1.0;
    }
    Vector3::new(x, y, z)
}

pub fn handle_device_event(input_manager: &mut InputManager, event: DeviceEvent) {
    if let DeviceEvent::MouseMotion { delta } = event {
        input_manager.mouse_delta = delta;
    }
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
