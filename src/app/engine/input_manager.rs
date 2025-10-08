use winit::event::KeyEvent;
use winit::keyboard::{Key, PhysicalKey};
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
    pub keybinds: Vec<Keybind>,
}

impl InputManager {
    pub fn new() -> Self {
        let forward = Keybind {
            key: PhysicalKey::Code(winit::keyboard::KeyCode::KeyA),
            input_type: KeybindInputType::Held,
            name: "move_forwards".to_string(),
        };

        let keybinds: Vec<Keybind> = vec![forward];

        Self {
            keys_held: Vec::new(),
            keybinds,
        }
    }
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
        }
    }
}

pub fn is_key_held(input_manager: &InputManager, key: PhysicalKey) -> bool {
    input_manager.keys_held.contains(&key)
}
pub fn is_keybind_triggered(input_manager: &InputManager, keybind: &Keybind) -> bool {
    match keybind.input_type {
        KeybindInputType::Pressed => is_key_held(input_manager, keybind.key),
        KeybindInputType::Released => is_key_held(input_manager, keybind.key),
        KeybindInputType::Held => is_key_held(input_manager, keybind.key),
    }
}
pub fn is_keybind_name_triggered(input_manager: &InputManager, keybind_name: String) -> bool {
    let keybind_index = input_manager
        .keybinds
        .iter()
        .position(|x| x.name == *keybind_name);

    if keybind_index.is_none() {
        println!("key doesnt align with a keybind");
        return false;
    }

    let keybind = &input_manager.keybinds[keybind_index.unwrap()];

    match keybind.input_type {
        KeybindInputType::Pressed => is_key_held(input_manager, keybind.key),
        KeybindInputType::Released => is_key_held(input_manager, keybind.key),
        KeybindInputType::Held => is_key_held(input_manager, keybind.key),
    }
}
