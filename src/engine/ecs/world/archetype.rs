use core::fmt;
use std::mem::MaybeUninit;

use aligned_vec::{AVec, RuntimeAlign};
use derive_more::{Deref, DerefMut, From};
use parking_lot::RwLock;
use smallvec::SmallVec;

use crate::{
    engine::ecs::{component::ComponentInfo, entity::Entity},
    utils::slotmap::{Key, SlotMap},
};

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
impl From<ArchetypeId> for Key {
    fn from(value: ArchetypeId) -> Self {
        value.0
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
pub struct Column {
    buffer: AVec<MaybeUninit<u8>, RuntimeAlign>,
    info: ComponentInfo,
}

impl Column {
    pub fn new(component_info: ComponentInfo) -> Self {
        Self {
            buffer: AVec::new(align_of::<ComponentInfo>()),
            info: component_info,
        }
    }
}

const ARCHETYPE_SAO: usize = 8;
pub struct Signature(SmallVec<[FieldId; ARCHETYPE_SAO]>);
impl Signature {
    pub fn new(fields: &[FieldId]) -> Self {
        // create new fields
        let mut fields = SmallVec::from(fields);

        // organise the fields and remove duplicates
        fields.sort();
        fields.dedup();

        Self(fields)
    }
}
