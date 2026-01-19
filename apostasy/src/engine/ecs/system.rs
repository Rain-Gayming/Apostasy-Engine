use crate::engine::ecs::World;

pub struct UpdateSystem {
    pub name: &'static str,
    pub func: fn(&mut World),
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
}
inventory::collect!(FixedUpdateSystem);

pub struct LateUpdateSystem {
    pub name: &'static str,
    pub func: fn(&mut World),
}
inventory::collect!(LateUpdateSystem);
