use crate::engine::ecs::{entity::Entity, world::core::Core};

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
        // info: ComponentInfo,
        // bytes: Box<[MaybeUninit<u8>]>,
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
    pub fn apply(&self, core: &mut Core) {
        match self.operation {
            Operation::Noop => {}
            Operation::Spawn(entity) => {
                core.initalize_entity_location(entity);
            }
            _ => (),
        }
    }

    pub fn spawn(entity: Entity) -> Self {
        Self {
            jump: 1,
            operation: Operation::Spawn(entity),
        }
    }
}
