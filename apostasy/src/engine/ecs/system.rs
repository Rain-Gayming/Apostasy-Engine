use crate::engine::ecs::World;

pub struct UpdateSystem {
    pub name: &'static str,
    pub func: fn(&mut World),
}

inventory::collect!(UpdateSystem);

pub struct StartSystem {
    pub name: &'static str,
    pub func: fn(&mut World),
}

inventory::collect!(StartSystem);
