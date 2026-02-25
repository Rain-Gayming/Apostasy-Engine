use egui::Context;

use crate::engine::{editor::EditorStorage, nodes::World, windowing::input_manager::InputManager};

pub struct UpdateSystem {
    pub name: &'static str,
    pub func: fn(&mut World),
    pub priority: u32,
}

inventory::collect!(UpdateSystem);

pub struct StartSystem {
    pub name: &'static str,
    pub func: fn(&mut World),
    pub priority: u32,
}

inventory::collect!(StartSystem);

pub struct FixedUpdateSystem {
    pub name: &'static str,
    pub func: fn(&mut World, delta: f32),
    pub priority: u32,
}
inventory::collect!(FixedUpdateSystem);

pub struct LateUpdateSystem {
    pub name: &'static str,
    pub func: fn(&mut World),
    pub priority: u32,
}
inventory::collect!(LateUpdateSystem);

pub struct UIFunction {
    pub name: &'static str,
    pub func: fn(&mut Context, &mut World),
    pub priority: u32,
}
inventory::collect!(UIFunction);

pub struct EditorUIFunction {
    pub name: &'static str,
    pub func: fn(&mut Context, &mut World, &mut EditorStorage),
    pub priority: u32,
}
inventory::collect!(EditorUIFunction);

pub struct InputSystem {
    pub name: &'static str,
    pub func: fn(&mut World, &mut InputManager),
    pub priority: u32,
}
inventory::collect!(InputSystem);
