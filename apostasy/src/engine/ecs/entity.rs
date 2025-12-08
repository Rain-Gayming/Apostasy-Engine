use derive_more::From;

use crate::{
    engine::ecs::{
        component::Component,
        world::{Mantle, World, archetype::ArchetypeId},
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

impl Entity {
    /// The null (uninitialized) state of a component
    pub fn null() -> Self {
        Self(Key::default())
    }

    /// Check to see if an entity is null (uninitialized)
    pub fn is_null(self) -> bool {
        self == Self::null()
    }

    /// Returns the raw key data of an entity
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

/// An entities location in an archetype
#[derive(Clone, Copy)]
pub struct EntityLocation {
    pub archetype: ArchetypeId,
    pub row: usize,
}

#[derive(Clone, Copy)]
pub struct View<'a> {
    pub entity: Entity,
    pub world: &'a World,
}

impl View<'_> {
    /// Returns the views index
    pub fn id(&self) -> Entity {
        self.entity
    }
    /// Inserts a component to the entity
    pub fn insert<C: Component>(self, component: C) -> Self {
        self.world.crust.mantle(|Mantle { core, .. }| {
            core.entity_location_locking(self.entity)
                .filter(|location| core.archetype_has(field.into(), location.archetype))
                .is_some()
        })
    }
}
