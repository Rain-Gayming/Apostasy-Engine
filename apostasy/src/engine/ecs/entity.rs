use derive_more::From;

use crate::{engine::ecs::world::archetype::ArchetypeId, utils::slotmap::Key};

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
}

/// An entities location in an archetype
pub struct EntityLocation {
    pub archetype: ArchetypeId,
    pub row: usize,
}
