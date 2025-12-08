use std::mem::{ManuallyDrop, MaybeUninit};

use crate::engine::ecs::{
    component::{Component, ComponentInfo},
    entity::Entity,
    world::archetype::FieldId,
};

#[derive(Debug)]
/// Operation for a command
enum Operation {
    Noop,
    Spawn(Entity),
    Despawn(Entity),
    Insert {
        info: ComponentInfo,
        bytes: Box<[MaybeUninit<u8>]>,
        entity: Entity,
    },
    Remove {
        field: FieldId,
        entity: Entity,
    },
}

pub struct Command {
    operation: Operation,
    jump: usize,
}
unsafe impl Send for Command {}

impl Default for Command {
    fn default() -> Self {
        Self {
            operation: Operation::Noop,
            jump: 1,
        }
    }
}

impl Command {
    pub fn insert<C: Component>(val: C, entity: Entity) -> Self {
        let leaked = ManuallyDrop::new(val);
        let bytes: &[MaybeUninit<u8>] = unsafe {
            std::slice::from_raw_parts((&raw const leaked).cast(), size_of::<C>()) //
        };
        // SAFETY: Safe because this is using static type info
        unsafe { Self::insert_bytes(C::info(), bytes.into(), entity) }
    }

    /// # SAFETY: unsafe as this is manual memory management
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
