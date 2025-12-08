use std::{collections::HashMap, mem::MaybeUninit};

use derive_more::{Deref, DerefMut};
use parking_lot::{MappedRwLockReadGuard, MappedRwLockWriteGuard, Mutex, RwLockReadGuard};

use crate::{
    engine::ecs::{
        entity::{Entity, EntityLocation},
        world::archetype::{Archetype, ArchetypeId, ColumnIndex, FieldId},
    },
    utils::slotmap::SlotMap,
};

#[derive(Deref, DerefMut, Default, Debug)]
pub struct FieldLocations(HashMap<ArchetypeId, ColumnIndex>);

pub struct Core {
    entity_index: Mutex<SlotMap<Entity, EntityLocation>>,
    field_index: HashMap<FieldId, FieldLocations>,
    archetypes: SlotMap<ArchetypeId, Archetype>,
    // signature index
}

impl Core {
    /// Gets an entities location in the Core
    pub fn entity_location(&mut self, entity: Entity) -> Option<EntityLocation> {
        let entity_index = self.entity_index.get_mut();
        entity_index.get(entity).copied()
    }
    /// Gets an entities location in the Core, returns a mutex
    pub fn entity_location_locking(&mut self, entity: Entity) -> Option<EntityLocation> {
        let entity_index = self.entity_index.lock();
        entity_index.get(entity).copied()
    }
}
