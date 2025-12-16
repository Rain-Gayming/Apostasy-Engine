use derive_more::From;

use crate::{
    engine::ecs::{
        component::Component,
        world::{World, archetype::ArchetypeId, commands::Command},
    },
    utils::slotmap::Key,
};

#[derive(Clone, Copy, Debug, From, PartialEq, Eq)]
pub struct Entity(pub Key);

impl From<Entity> for Key {
    fn from(value: Entity) -> Self {
        value.0
    }
}

/// An entities location in an archetype
pub struct EntityLocation {
    pub archetype: ArchetypeId,
    pub row: usize,
}

impl Entity {
    pub fn raw(self) -> u64 {
        self.0.raw()
    }
    pub fn from_raw(val: u64) -> Self {
        Self(Key::from_raw(val))
    }

    /// # Safety
    /// Should never be called manually
    pub unsafe fn from_offset(val: u32) -> Self {
        Self(Key {
            index: val,
            generation: 1,
        })
    }
}

#[derive(Clone, Copy)]
pub struct View<'a> {
    pub entity: Entity,
    pub world: &'a World,
}

impl View<'_> {
    pub fn id(&self) -> Entity {
        self.entity
    }

    pub fn insert<C: Component>(self, component: C) -> Self {
        self.world.crust.mantle(|mantle| {
            mantle.enqueue(Command::insert(component, self.entity));
        });
        self
    }
}
