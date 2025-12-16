use std::mem::{ManuallyDrop, MaybeUninit};

use crate::engine::ecs::{
    component::{Component, ComponentInfo},
    entity::Entity,
    world::core::Core,
};

/// What type of operation is being run
#[derive(Debug)]
pub enum Operation {
    /// No operation
    Noop,
    /// Spawning an entity
    Spawn(Entity),
    /// Despawning an entity
    Despawn(Entity),

    /// Inserting a component to an entity
    Insert {
        info: ComponentInfo,
        bytes: Box<[MaybeUninit<u8>]>,
        entity: Entity,
    },
}

#[derive(Debug)]
pub struct Command {
    pub operation: Operation,
    pub jump: usize,
}

impl Default for Command {
    fn default() -> Self {
        Self {
            operation: Operation::Noop,
            jump: 1,
        }
    }
}

impl Command {
    pub fn apply(self, core: &mut Core) {
        match self.operation {
            Operation::Noop => {}
            Operation::Spawn(entity) => {
                core.initialize_entity_location(entity);
            }
            _ => (),
        };
    }

    /// Create a command for spawning an entity
    pub fn spawn(entity: Entity) -> Self {
        Self {
            jump: 1,
            operation: Operation::Spawn(entity),
        }
    }

    pub fn insert<C: Component>(val: C, entity: Entity) -> Self {
        let leaked = ManuallyDrop::new(val);
        let bytes: &[MaybeUninit<u8>] = unsafe {
            std::slice::from_raw_parts((&raw const leaked).cast(), size_of::<C>()) //
        };
        // SAFETY: Safe because this is using static type info
        unsafe { Self::insert_bytes(C::info(), bytes.into(), entity) }
    }

    pub unsafe fn insert_bytes(
        info: ComponentInfo,
        bytes: Box<[MaybeUninit<u8>]>,
        entity: Entity,
    ) -> Self {
        Self {
            jump: 1,
            operation: Operation::Insert {
                info,
                bytes,
                entity,
            },
        }
    }
}
