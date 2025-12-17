use aligned_vec::{AVec, RuntimeAlign};
use derive_more::From;

use crate::{
    engine::ecs::{component::ComponentInfo, entity::Entity},
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

/// Reference to a non-entity ID (component, resource ect)
pub struct FieldId(u64);

pub struct Column {
    pub info: ComponentInfo,
    pub buffer: AVec<u8, RuntimeAlign>,
}

impl Column {
    pub fn new(component_info: ComponentInfo) -> Self {
        Self {
            buffer: AVec::new(component_info.align),
            info: component_info,
        }
    }
}

/// The fields (typically components) in a vec
pub struct Signature(Vec<FieldId>);
