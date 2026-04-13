use anyhow::Result;

use crate::objects::world::World;

/// A system that happens every frame
pub struct UpdateSystem {
    pub name: &'static str,
    pub func: fn(&mut World) -> Result<()>,
    pub priority: u32,
}
inventory::collect!(UpdateSystem);

/// A system that happens once at the start of the application
pub struct StartSystem {
    pub name: &'static str,
    pub func: fn(&mut World) -> Result<()>,
    pub priority: u32,
}
inventory::collect!(StartSystem);

/// A system that happens x amount of times per second
pub struct FixedUpdateSystem {
    pub name: &'static str,
    pub func: fn(&mut World, delta: f32) -> Result<()>,
    pub priority: u32,
}
inventory::collect!(FixedUpdateSystem);

/// A system that happens at the end over every frame
pub struct LateUpdateSystem {
    pub name: &'static str,
    pub func: fn(&mut World) -> Result<()>,
    pub priority: u32,
}
inventory::collect!(LateUpdateSystem);
