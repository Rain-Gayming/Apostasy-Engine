use crate::{log, log_warn};
use cgmath::{Vector2, Vector3};
use egui::ahash::HashMap;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, path::Path};
use winit::{
    dpi::PhysicalPosition,
    event::{DeviceEvent, MouseButton, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
};
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KeyAction {
    Press,
    Release,
    Hold,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyBind {
    pub key: PhysicalKey,
    pub action: KeyAction,
    pub name: String,
}
impl KeyBind {
    pub fn new(key: PhysicalKey, action: KeyAction, name: String) -> Self {
        Self { key, action, name }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MouseBind {
    pub key: MouseButton,
    pub action: KeyAction,
    pub name: String,
}
impl MouseBind {
    pub fn new(key: MouseButton, action: KeyAction, name: String) -> Self {
        Self { key, action, name }
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
        self.serialize_input_manager().unwrap();
    }

    /// Registers a key, use:
    /// ```rust
    ///     world.with_resource_mut(|self. &mut InputManager| {
    ///         register_key(self. KeyBind::new(PhysicalKey::Code(KeyCode::KeyW), KeyAction::Hold), "forward");
    ///     });
    /// ```
    pub fn register_keybind(&mut self, key: KeyBind) {
        log!("registering keybind: {}", key.name.clone());
        if self.keybinds.contains_key(&key.name) {
            log_warn!("keybind already exists: {}", key.name);
            return;
        }
        self.keybinds.insert(key.name.clone(), key);
        self.serialize_input_manager().unwrap();
    }

    pub fn register_mousebind(&mut self, key: MouseBind) {
        log!("registering mousebind: {}", key.name.clone());
        if self.mouse_keybinds.contains_key(&key.name) {
            log_warn!("mousebind already exists: {}", key.name);
            return;
        }
        self.mouse_keybinds.insert(key.name.clone(), key);
        self.serialize_input_manager().unwrap();
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

    pub fn serialize_input_manager(&self) -> Result<(), std::io::Error> {
        let keybinds = self.serialize_bindings().unwrap();
        let path = format!("{}/{}.yaml", ENGINE_INPUT_SAVE_PATH, "input_manager");
        if !Path::new(&path).exists() {
            std::fs::create_dir_all(ENGINE_INPUT_SAVE_PATH)?;
        }
        std::fs::write(path, keybinds)
    }

    pub fn deserialize_input_manager(&mut self) -> Result<(), std::io::Error> {
        let path = format!("{}/{}.yaml", ENGINE_INPUT_SAVE_PATH, "input_manager");

        let contents = std::fs::read_to_string(path)?;

        let (key_bindings, mouse_bindings) = self
            .deserialize_bindings(&contents)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        self.keybinds = key_bindings;
        self.mouse_keybinds = mouse_bindings;

        Ok(())
    }
    pub fn serialize_bindings(&self) -> Result<String, serde_yaml::Error> {
        let key_binds: Vec<serde_yaml::Value> = self
            .keybinds
            .iter()
            .map(|(name, bind)| {
                serde_yaml::to_value(serde_yaml::Mapping::from_iter([
                    (
                        serde_yaml::Value::String("name".into()),
                        serde_yaml::to_value(name).unwrap(),
                    ),
                    (
                        serde_yaml::Value::String("bind".into()),
                        serde_yaml::to_value(bind).unwrap(),
                    ),
                ]))
                .unwrap()
            })
            .collect();

        let mouse_binds: Vec<serde_yaml::Value> = self
            .mouse_keybinds
            .iter()
            .map(|(name, bind)| {
                serde_yaml::to_value(serde_yaml::Mapping::from_iter([
                    (
                        serde_yaml::Value::String("name".into()),
                        serde_yaml::to_value(name).unwrap(),
                    ),
                    (
                        serde_yaml::Value::String("bind".into()),
                        serde_yaml::to_value(bind).unwrap(),
                    ),
                ]))
                .unwrap()
            })
            .collect();

        let mut output = serde_yaml::Mapping::new();
        output.insert(
            serde_yaml::Value::String("key_bindings".into()),
            serde_yaml::to_value(key_binds).unwrap(),
        );
        output.insert(
            serde_yaml::Value::String("mouse_bindings".into()),
            serde_yaml::to_value(mouse_binds).unwrap(),
        );

        serde_yaml::to_string(&output)
    }

    pub fn deserialize_bindings(
        &self,
        contents: &str,
    ) -> Result<(HashMap<String, KeyBind>, HashMap<String, MouseBind>), serde_yaml::Error> {
        let raw: serde_yaml::Value = serde_yaml::from_str(contents)?;

        let key_bindings = raw["key_bindings"]
            .as_sequence()
            .map(|seq| {
                seq.iter()
                    .filter_map(|entry| {
                        let name = entry["name"].as_str()?.to_string();
                        let bind = match serde_yaml::from_value::<KeyBind>(entry["bind"].clone()) {
                            Ok(b) => b,
                            Err(e) => {
                                eprintln!("failed to deserialize KeyBind: {e}");
                                return None;
                            }
                        };
                        Some((name, bind))
                    })
                    .collect()
            })
            .unwrap_or_default();

        let mouse_bindings = raw["mouse_bindings"]
            .as_sequence()
            .map(|seq| {
                seq.iter()
                    .filter_map(|entry| {
                        let name = entry["name"].as_str()?.to_string();
                        let bind = match serde_yaml::from_value::<MouseBind>(entry["bind"].clone())
                        {
                            Ok(b) => b,
                            Err(e) => {
                                eprintln!("failed to deserialize MouseBind: {e}");
                                return None;
                            }
                        };
                        Some((name, bind))
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok((key_bindings, mouse_bindings))
    }
}

const ENGINE_INPUT_SAVE_PATH: &str = "res/input";
