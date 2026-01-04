use std::any::type_name;

use derive_more::From;

use crate::{
    engine::ecs::{
        World,
        archetype::{ArchetypeId, RowIndex},
        command::Command,
        component::Component,
    },
    utils::slotmap::Key,
};

/// The location of an entity in an archetype
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EntityLocation {
    pub archetype: ArchetypeId,
    pub row: RowIndex,
}

impl EntityLocation {
    pub fn uninitialized() -> Self {
        Self {
            archetype: ArchetypeId::empty_archetype(),
            row: RowIndex(usize::MAX),
        }
    }
}

/// They key for an entity
#[derive(Clone, Copy, Debug, From, PartialEq, Eq)]
pub struct Entity(Key);

impl From<Entity> for Key {
    fn from(value: Entity) -> Self {
        value.0
    }
}

impl Entity {
    /// # Safety
    /// Should never be called manually
    pub unsafe fn from_offset(val: u32) -> Self {
        Self(Key {
            index: val,
            generation: 1,
        })
    }

    pub fn raw(self) -> u64 {
        self.0.raw()
    }

    pub fn from_raw(val: u64) -> Self {
        Self(Key::from_raw(val))
    }
}

#[derive(Clone, Copy)]
pub struct EntityView<'a> {
    pub entity: Entity,
    pub world: &'a World,
}

impl EntityView<'_> {
    pub fn id(&self) -> Entity {
        self.entity
    }
    pub fn insert<C: Component>(self, component: C) -> Self {
        self.world.crust.mantle(|mantle| {
            mantle.queue_command(Command::insert(component, self.entity));
        });
        self
    }
}
