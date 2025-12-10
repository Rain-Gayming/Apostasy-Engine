use std::collections::HashMap;

use derive_more::{Deref, DerefMut};
use parking_lot::Mutex;

use crate::{
    engine::ecs::{
        entity::Entity,
        world::archetype::{ArchetypeId, ColumnIndex, FieldId, RowIndex},
    },
    utils::slotmap::SlotMap,
};

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

#[derive(Deref, DerefMut, Default, Debug)]
pub struct FieldLocations(HashMap<ArchetypeId, ColumnIndex>);

pub struct Core {
    entity_index: Mutex<SlotMap<Entity, EntityLocation>>,
    field_index: HashMap<FieldId, FieldLocations>,
    // archetypes: SlotMap<ArchetypeId, Archetype>,
    // signature index
    // archetypes
}
