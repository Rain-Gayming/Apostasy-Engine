use std::{
    cell::{Cell, UnsafeCell},
    process::Command,
    sync::Arc,
};

use crate::engine::ecs::world::core::Core;

pub mod archetype;
pub mod commands;
pub mod core;

/// Wrapper for Crust
pub struct World {
    pub crust: Arc<Crust>,
}

/// Wrapper for Crust
pub struct Crust {
    pub mantle: UnsafeCell<Mantle>,
}

/// Wrapper for Core
pub struct Mantle {
    pub core: Core,
    pub commands: Cell<Vec<Command>>,
}
