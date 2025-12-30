use std::mem::MaybeUninit;

use aligned_vec::{AVec, RuntimeAlign};
use derive_more::From;

use crate::{
    engine::ecs::{
        component::{Component, ComponentInfo},
        entity::{self, Entity},
    },
    utils::slotmap::Key,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RowIndex(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColumnIndex(pub usize);

#[derive(Debug, Clone, Copy, From, PartialEq, Eq)]
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

/// A data set that stores entities and components
pub struct Archetype {
    pub entities: Vec<Entity>,
    pub columns: Vec<Column>,
    pub signature: Signature,
    // TODO: add edges
}

impl Archetype {
    /// Add an entity and it's components to the archetype
    pub fn insert(&mut self, /*data: Vec<Box<dyn Component>>, */ entity: Entity) {
        // add the entity to the archetyp
        self.entities.insert(self.entities.len(), entity);

        // add the components to their columns
        for column in self.columns.iter_mut() {}
    }
}

/// Reference to a non-entity ID (component, resource ect)
#[derive(Debug, Clone, Copy, From, PartialEq, Eq)]
pub struct FieldId(u64);

pub struct Column {
    pub info: ComponentInfo,
    pub buffer: AVec<MaybeUninit<u8>, RuntimeAlign>,
}

impl Column {
    pub fn new(component_info: ComponentInfo) -> Self {
        Self {
            buffer: AVec::new(component_info.align),
            info: component_info,
        }
    }

    /// Gets the last row and swaps it with an empty row to colapse down the vec
    pub fn swap_with_last(&mut self, RowIndex(row): RowIndex) {
        if row + 1 < self.no_chunks() {
            let (left, right) = self.buffer.split_at_mut((row + 1) * self.info.size);
            left[row * self.info.size..].swap_with_slice(right);
        }
    }

    /// Returns the size of a part of the buffer,
    /// if the component has no size it returns 0
    pub fn no_chunks(&self) -> usize {
        if self.info.size == 0 {
            0
        } else {
            self.buffer.len() / self.info.size
        }
    }
}

/// The components in a vec
pub struct Signature(Vec<FieldId>);
