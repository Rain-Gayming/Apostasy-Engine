use std::mem::MaybeUninit;

use aligned_vec::{AVec, RuntimeAlign};
use derive_more::{Deref, DerefMut, From};
use parking_lot::RwLock;

use crate::{
    engine::ecs::{
        component::{ComponentInfo, ComponentVec},
        entity::Entity,
    },
    utils::slotmap::Key,
};

#[derive(Clone, Copy, Debug, From, PartialEq, Eq, Hash)]
pub struct ArchetypeId(pub Key);

pub struct Archetype {
    pub signature: ComponentVec,
    pub entities: Vec<Entity>,
    pub columns: Vec<RwLock<Column>>,
}

#[derive(Debug)]
pub struct Column {
    buffer: AVec<MaybeUninit<u8>, RuntimeAlign>,
    info: ComponentInfo,
}

#[derive(Clone, Copy, Deref, DerefMut, Debug)]
pub struct ColumnIndex(pub usize);

#[derive(Clone, Copy, Deref, DerefMut, Debug, PartialEq, Eq)]
pub struct RowIndex(pub usize);

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
