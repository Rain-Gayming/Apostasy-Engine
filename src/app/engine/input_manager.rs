use winit::event::KeyEvent;
use winit::keyboard::PhysicalKey;
#[derive(Clone, Copy)]
pub enum KeybindInputType {
    Pressed,
    Released,
    Held,
}

#[derive(Clone)]
pub struct Keybind {
    pub key: PhysicalKey,
    pub input_type: KeybindInputType,
    pub name: String,
}

pub struct InputManager {
    pub keys_held: Vec<PhysicalKey>,
    pub keys_to_ignore: Vec<PhysicalKey>,
    pub keys_released: Vec<PhysicalKey>,
    pub keybinds: Vec<Keybind>,
    pub mouse_delta: [f64; 2],
}

impl Default for InputManager {
    fn default() -> Self {
        let forward = Keybind {
            key: PhysicalKey::Code(winit::keyboard::KeyCode::KeyW),
            input_type: KeybindInputType::Held,
            name: "move_forwards".to_string(),
        };

        let backwards = Keybind {
            key: PhysicalKey::Code(winit::keyboard::KeyCode::KeyS),
            input_type: KeybindInputType::Held,
            name: "move_backwards".to_string(),
        };

        let left = Keybind {
            key: PhysicalKey::Code(winit::keyboard::KeyCode::KeyA),
            input_type: KeybindInputType::Held,
            name: "move_left".to_string(),
        };

        let right = Keybind {
            key: PhysicalKey::Code(winit::keyboard::KeyCode::KeyD),
            input_type: KeybindInputType::Held,
            name: "move_right".to_string(),
        };

        let jump = Keybind {
            key: PhysicalKey::Code(winit::keyboard::KeyCode::Space),
            input_type: KeybindInputType::Held,
            name: "move_jump".to_string(),
        };

        let crouch = Keybind {
            key: PhysicalKey::Code(winit::keyboard::KeyCode::ControlLeft),
            input_type: KeybindInputType::Held,
            name: "move_crouch".to_string(),
        };

        let pause = Keybind {
            key: PhysicalKey::Code(winit::keyboard::KeyCode::Escape),
            input_type: KeybindInputType::Released,
            name: "game_pause".to_string(),
        };

        let keybinds: Vec<Keybind> = vec![forward, backwards, left, right, jump, crouch, pause];

        Self {
            keys_held: Vec::new(),
            keys_to_ignore: Vec::new(),
            keys_released: Vec::new(),
            keybinds,
            mouse_delta: [0.0, 0.0],
        }
    }
}

pub fn update_mouse_delta(input_manager: &mut InputManager, delta: [f64; 2]) {
    input_manager.mouse_delta = delta;
}

pub fn update_or_add_keybind(input_manager: &mut InputManager, keybind: Keybind) {
    let keybind_index = input_manager
        .keybinds
        .iter()
        .position(|x| x.name == *keybind.name);

    if keybind_index.is_none() {
        println!("adding new keybind {}", keybind.name);
        input_manager.keybinds.push(keybind);
    } else {
        println!("updaing keybind: {}", keybind.name);
        input_manager.keybinds[keybind_index.unwrap()] = keybind;
    }
}

pub fn process_keyboard_input(input_manager: &mut InputManager, event: &KeyEvent) {
    let physical_key = event.physical_key;

    match event.state {
        winit::event::ElementState::Pressed => {
            if !input_manager.keys_held.contains(&physical_key) {
                input_manager.keys_held.push(physical_key);
            }
        }
        winit::event::ElementState::Released => {
            input_manager.keys_held.retain(|&key| key != physical_key);
            input_manager.keys_released.push(physical_key);
        }
    }
}

pub fn is_key_pressed(input_manager: &mut InputManager, key: PhysicalKey) -> bool {
    if input_manager.keys_held.contains(&key) && !input_manager.keys_to_ignore.contains(&key) {
        input_manager.keys_to_ignore.push(key);
        input_manager.keys_held.retain(|&x| x != key);
        true
    } else {
        false
    }
}

pub fn is_key_released(input_manager: &mut InputManager, key: PhysicalKey) -> bool {
    if input_manager.keys_released.contains(&key) && !input_manager.keys_to_ignore.contains(&key) {
        input_manager.keys_released.retain(|&x| x != key);
        true
    } else {
        false
    }
}
pub fn is_key_held(input_manager: &InputManager, key: PhysicalKey) -> bool {
    input_manager.keys_held.contains(&key)
}

pub fn is_keybind_triggered(input_manager: &mut InputManager, keybind: &Keybind) -> bool {
    match keybind.input_type {
        KeybindInputType::Pressed => is_key_pressed(input_manager, keybind.key),
        KeybindInputType::Released => is_key_released(input_manager, keybind.key),
        KeybindInputType::Held => is_key_held(input_manager, keybind.key),
    }
}
pub fn is_keybind_name_triggered(input_manager: &mut InputManager, keybind_name: String) -> bool {
    let keybind_index = input_manager
        .keybinds
        .iter()
        .position(|x| x.name == *keybind_name);

    if keybind_name.is_empty() {
        return false;
    }

    if keybind_index.is_none() {
        println!("key doesnt align with a keybind");
        return false;
    }

    let keybind = &input_manager.keybinds[keybind_index.unwrap()];

    match keybind.input_type {
        KeybindInputType::Pressed => is_key_pressed(input_manager, keybind.key),
        KeybindInputType::Released => is_key_released(input_manager, keybind.key),
        KeybindInputType::Held => is_key_held(input_manager, keybind.key),
    }
}
