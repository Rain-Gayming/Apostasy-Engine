use anyhow::Result;

use crate::objects::world::World;

pub trait HasPriority {
    fn priority(&self) -> u32;
}

/// A system that happens every frame
pub struct UpdateSystem {
    pub name: &'static str,
    pub func: fn(&mut World) -> Result<()>,
    pub priority: u32,
}
inventory::collect!(UpdateSystem);
impl HasPriority for UpdateSystem {
    fn priority(&self) -> u32 {
        self.priority
    }
}

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

impl HasPriority for FixedUpdateSystem {
    fn priority(&self) -> u32 {
        self.priority
    }
}
/// A system that happens at the end over every frame
pub struct LateUpdateSystem {
    pub name: &'static str,
    pub func: fn(&mut World) -> Result<()>,
    pub priority: u32,
}
inventory::collect!(LateUpdateSystem);

impl HasPriority for LateUpdateSystem {
    fn priority(&self) -> u32 {
        self.priority
    }
}
