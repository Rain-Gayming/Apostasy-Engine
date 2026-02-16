use egui::Context;

use crate::engine::ecs::World;

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
