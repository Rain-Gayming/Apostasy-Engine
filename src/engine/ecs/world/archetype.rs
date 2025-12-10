use std::mem::MaybeUninit;

use aligned_vec::{AVec, RuntimeAlign};
use derive_more::{Deref, DerefMut, From};
use parking_lot::RwLock;
use smallvec::SmallVec;

use crate::{engine::ecs::entity::Entity, utils::slotmap::Key};
const ARCHETYPE_SAO: usize = 8;

#[derive(Clone, Copy, Deref, DerefMut, Debug)]
pub struct ColumnIndex(pub usize);

#[derive(Clone, Copy, Deref, DerefMut, Debug, PartialEq, Eq)]
pub struct RowIndex(pub usize);

#[derive(Clone, Copy, Debug, From, PartialEq, Eq, Hash)]
pub struct ArchetypeId(pub Key);

impl ArchetypeId {
    pub fn empty_archetype() -> ArchetypeId {
        Self(Key {
            index: 0,
            generation: 1,
        })
    }
}

/// Component or pair
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FieldId(pub u64);

impl From<Entity> for FieldId {
    fn from(entity: Entity) -> Self {
        Self(entity.raw() & u32::MAX as u64)
    }
}

impl FieldId {
    pub fn as_entity(&self) -> Option<Entity> {
        Some(Entity::from_raw(self.0))
    }
}

#[derive(Debug, Default)]
pub struct Archetype {
    pub entities: Vec<Entity>,
    pub columns: Vec<RwLock<Column>>,
    // pub edges: HashMap<FieldId, ArchetypeEdge>,
}

#[derive(Debug)]
pub(crate) struct Column {
    buffer: AVec<MaybeUninit<u8>, RuntimeAlign>,
    // info: ComponentInfo,
}
