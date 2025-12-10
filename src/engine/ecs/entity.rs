use derive_more::From;

use crate::{engine::ecs::world::archetype::ArchetypeId, utils::slotmap::Key};

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
}
