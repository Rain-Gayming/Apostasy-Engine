use std::mem::{ManuallyDrop, MaybeUninit};

use crate::engine::ecs::{
    World,
    component::{Component, ComponentInfo},
    entity::{self, Entity},
};

pub enum CommandOperation {
    Null,
    Spawn(Entity),
    Despawn(Entity),
    Insert {
        info: ComponentInfo,
        bytes: Box<[MaybeUninit<u8>]>,
        entity: Entity,
    },
}
pub struct Command {
    pub operation: CommandOperation,
    pub jump: usize,
}

unsafe impl Send for Command {}

impl Default for Command {
    fn default() -> Self {
        Self {
            operation: CommandOperation::Null,
            jump: 1,
        }
    }
}

impl Command {
    pub fn apply(self, world: &mut World) {
        use CommandOperation::*;
        match self.operation {
            Null => {}
            Spawn(entity) => {
                world.spawn_entity(entity);
            }
            // TODO:
            // Despawn(entity) => {
            //     world.despawn(entity);
            // }
            Insert {
                info,
                bytes,
                entity,
            } => {
                unsafe { world.insert_component_bytes(info, &bytes, entity) };
            }

            _ => {
                println!("Command has no function");
            }
        }
    }

    pub fn spawn(entity: Entity) -> Self {
        Self {
            jump: 1,
            operation: CommandOperation::Spawn(entity),
        }
    }

    pub fn despawn(entity: Entity) -> Self {
        Self {
            jump: 1,
            operation: CommandOperation::Despawn(entity),
        }
    }

    pub fn insert<C: Component>(component: C, entity: Entity) -> Self {
        let leaked = ManuallyDrop::new(component);
        let bytes: &[MaybeUninit<u8>] = unsafe {
            std::slice::from_raw_parts((&raw const leaked).cast(), size_of::<C>()) //
        };
        unsafe { Self::insert_bytes(C::info(), bytes.into(), entity) }
    }

    pub unsafe fn insert_bytes(
        info: ComponentInfo,
        bytes: Box<[MaybeUninit<u8>]>,
        entity: Entity,
    ) -> Self {
        Self {
            jump: 1,
            operation: CommandOperation::Insert {
                info,
                bytes,
                entity,
            },
        }
    }
}
