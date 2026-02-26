use crate::log;
use cgmath::{Vector2, Vector3};
use egui::ahash::HashMap;
use std::collections::HashSet;
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

pub struct MouseBind {
    pub key: MouseButton,
    pub action: KeyAction,
}
impl MouseBind {
    pub fn new(key: MouseButton, action: KeyAction) -> Self {
        Self { key, action }
    }
}

#[derive(Default)]
pub struct InputManager {
    keybinds: HashMap<String, KeyBind>,
    mouse_keybinds: HashMap<String, MouseBind>,
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

impl InputManager {
    /// Rebinds a key, use:
    /// ```rust
    ///     world.with_resource_mut(|self. &mut InputManager| {
    ///         rebind_key(self. KeyBind::new(PhysicalKey::Code(KeyCode::KeyW), KeyAction::Hold), "forward");
    ///     });
    /// ```
    pub fn rebind_key(&mut self, key: KeyBind, name: &str) {
        self.keybinds.remove(name);
        self.keybinds.insert(name.to_string(), key);
    }

    /// Registers a key, use:
    /// ```rust
    ///     world.with_resource_mut(|self. &mut InputManager| {
    ///         register_key(self. KeyBind::new(PhysicalKey::Code(KeyCode::KeyW), KeyAction::Hold), "forward");
    ///     });
    /// ```
    pub fn register_keybind(&mut self, key: KeyBind, name: &str) {
        log!("registering keybind: {}", name);
        self.keybinds.insert(name.to_string(), key);
    }

    pub fn register_mousebind(&mut self, key: MouseBind, name: &str) {
        log!("registering mousebind: {}", name);
        self.mouse_keybinds.insert(name.to_string(), key);
    }

    /// Checks if a key is active, use:
    /// ```rust
    ///     world.with_resource_mut(|self. &mut InputManager| {
    ///         if is_keybind_active(self. "forward") {
    ///             log!("forward is active");
    ///         }
    ///     });
    /// ```
    pub fn is_keybind_active(&self, name: &str) -> bool {
        let key = self.keybinds.get(name);
        if key.is_none() {
            return false;
        }
        let key = key.unwrap();
        match key.action {
            KeyAction::Press => self.keys_pressed.contains(&key.key),
            KeyAction::Release => self.keys_released.contains(&key.key),
            KeyAction::Hold => self.keys_held.contains(&key.key),
        }
    }

    pub fn is_mousebind_active(&self, name: &str) -> bool {
        let key = self.mouse_keybinds.get(name);
        if key.is_none() {
            return false;
        }
        let key = key.unwrap();
        match key.action {
            KeyAction::Press => self.mouse_pressed.contains(&key.key),
            KeyAction::Release => self.mouse_released.contains(&key.key),
            KeyAction::Hold => self.mouse_held.contains(&key.key),
        }
    }

    // #[late_update]
    pub fn clear_actions(&mut self) {
        // self.keys_pressed.clear();
        // self.keys_released.clear();
        self.mouse_pressed.clear();
        self.mouse_released.clear();
        self.mouse_delta = (0.0, 0.0);
    }

    /// Calculates the input vector for 2D movement, use:
    /// ```rust
    ///     world.with_resource_mut(|self. &mut InputManager| {
    ///         let direction = input_vector_2d(
    ///             self.
    ///             "left",
    ///             "right",
    ///             "up",
    ///             "down",
    ///         );
    ///     });
    /// ```
    pub fn input_vector_2d(&self, left: &str, right: &str, up: &str, down: &str) -> Vector2<f32> {
        let mut x = 0.0;
        let mut y = 0.0;
        if self.is_keybind_active(left) {
            x += 1.0;
        }
        if self.is_keybind_active(right) {
            x -= 1.0;
        }
        if self.is_keybind_active(up) {
            y += 1.0;
        }
        if self.is_keybind_active(down) {
            y -= 1.0;
        }
        Vector2::new(x, y)
    }

    /// Calculates the input vector for 3D movement, use:
    /// ```rust
    ///     world.with_resource_mut(|self. &mut InputManager| {
    ///         let direction = input_vector_3d(
    ///             self.
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
        &self,
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
        if self.is_keybind_active(x_pos) {
            x += 1.0;
        }
        if self.is_keybind_active(x_neg) {
            x -= 1.0;
        }
        if self.is_keybind_active(y_pos) {
            y += 1.0;
        }
        if self.is_keybind_active(y_neg) {
            y -= 1.0;
        }
        if self.is_keybind_active(z_pos) {
            z += 1.0;
        }
        if self.is_keybind_active(z_neg) {
            z -= 1.0;
        }
        Vector3::new(x, y, z)
    }

    pub fn handle_device_event(&mut self, event: DeviceEvent) {
        if let DeviceEvent::MouseMotion { delta } = event {
            self.mouse_delta = delta;
        }
    }

    pub fn handle_input_event(&mut self, event: WindowEvent) {
        match event {
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state.is_pressed() {
                    self.keys_pressed.insert(event.physical_key);
                    self.keys_held.insert(event.physical_key);
                } else {
                    self.keys_released.insert(event.physical_key);
                    self.keys_held.remove(&event.physical_key);
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                println!("mouse input: {:?}", button);
                if state.is_pressed() {
                    self.mouse_pressed.insert(button);
                    self.mouse_held.insert(button);
                } else {
                    self.mouse_released.insert(button);
                    self.mouse_held.remove(&button);
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                let delta = (
                    position.x - self.mouse_position.x,
                    position.y - self.mouse_position.y,
                );
                self.mouse_delta = delta;
                self.mouse_position = position;
            }
            _ => {}
        }
    }
}
