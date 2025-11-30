use std::mem::MaybeUninit;

use crate::{engine::ecs::entity::Entity, utils::slotmap::Key};
use aligned_vec::{AVec, RuntimeAlign};
use derive_more::{Deref, DerefMut, From};
use smallvec::SmallVec;

pub const ARCHETYPE_SAO: usize = 8;

/// A pointer to an archetype
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
impl From<ArchetypeId> for Key {
    fn from(value: ArchetypeId) -> Self {
        value.0
    }
}

/// A pointer to an objects position in a column
#[derive(Clone, Copy, Deref, DerefMut, Debug)]
pub struct ColumnIndex(pub usize);

/// A pointer to an objects position in a row
#[derive(Clone, Copy, Deref, DerefMut, Debug, PartialEq, Eq)]
pub struct RowIndex(pub usize);

/// A pointer to a component or pair
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

/// A collection of component or pair types stored in a vec for archetype referencing
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Signature(SmallVec<[FieldId; ARCHETYPE_SAO]>);

impl Signature {
    pub fn new(fields: &[FieldId]) -> Self {
        let mut fields = SmallVec::from(fields);
        fields.sort();
        fields.dedup();
        Self(fields)
    }

    pub fn contains(&self, field: FieldId) -> bool {
        self.0.binary_search(&field).is_ok()
    }

    pub fn with(mut self, field: FieldId) -> Self {
        if let Err(n) = self.0.binary_search(&field) {
            self.0.insert(n, field);
        }
        self
    }

    pub fn without(mut self, field: FieldId) -> Self {
        if let Ok(n) = self.0.binary_search(&field) {
            self.0.remove(n);
        };
        self
    }
}

#[derive(Debug)]
pub(crate) struct Column {
    buffer: AVec<MaybeUninit<u8>, RuntimeAlign>,
    // info: ComponentInfo,
}
