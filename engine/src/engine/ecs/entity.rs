use derive_more::From;

use crate::{
    engine::ecs::world::archetype::{ArchetypeId, RowIndex},
    utils::slotmap::Key,
};

#[derive(Debug, PartialEq, Clone, Copy, Eq)]
pub struct EntityLocation {
    pub archetype: ArchetypeId,
    pub row: RowIndex,
}

impl EntityLocation {
    pub fn uninitalized() -> Self {
        Self {
            archetype: ArchetypeId::empty_archetype(),
            row: RowIndex(usize::MAX),
        }
    }
}

#[derive(Debug, Clone, Copy, From, PartialEq, Eq)]
pub struct Entity(pub Key);

impl From<Entity> for Key {
    fn from(value: Entity) -> Self {
        value.0
    }
}
