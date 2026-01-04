use std::{collections::HashMap, fmt::Debug, mem::MaybeUninit};

use aligned_vec::{AVec, RuntimeAlign};
use derive_more::{Deref, DerefMut, From};
use parking_lot::RwLock;
use smallvec::SmallVec;

use crate::{
    engine::ecs::{
        Entity,
        component::{ComponentId, ComponentInfo},
    },
    utils::slotmap::{Key, Slot},
};

/// A data type that contains
///     - entities
///     - component data
///     - component types
#[derive(Debug, Default)]
pub struct Archetype {
    pub signature: Signature,
    pub entities: Vec<Entity>,
    pub columns: Vec<RwLock<Column>>,
    pub edges: HashMap<ComponentId, ArchetypeEdge>,
}

impl Debug for Slot<Archetype> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Archetype")
            .field("signature", &self.data.as_ref().unwrap().signature)
            .field("entities", &self.data.as_ref().unwrap().entities)
            .finish()
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ArchetypeEdge {
    pub add: Option<ArchetypeId>,
    pub remove: Option<ArchetypeId>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, From, Hash)]
pub struct ArchetypeId(pub Key);
impl ArchetypeId {
    pub fn empty_archetype() -> Self {
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

/// The position of data in an archetype row
#[derive(Clone, Deref, DerefMut, Copy, Debug, PartialEq, Eq)]
pub struct RowIndex(pub usize);
/// The position of data in an archetype column
#[derive(Clone, Deref, DerefMut, Copy, Debug, PartialEq, Eq)]
pub struct ColumnIndex(pub usize);

/// Data type that holds the types of components in an archetype
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Signature(pub SmallVec<[ComponentId; 8]>);

impl Signature {
    pub fn new(fields: &[ComponentId]) -> Self {
        let mut fields = SmallVec::from(fields);
        fields.sort();
        fields.dedup();
        Self(fields)
    }

    /// Checks if a signature contains a component
    pub fn contains(&self, component: ComponentId) -> bool {
        self.0.binary_search(&component).is_ok()
    }

    /// Creates a new signature with the new Id
    pub fn with(mut self, component: ComponentId) -> Self {
        if let Err(n) = self.0.binary_search(&component) {
            self.0.insert(n, component);
        }
        self
    }

    /// Creates a new signature without the Id
    pub fn without(mut self, field: ComponentId) -> Self {
        if let Ok(n) = self.0.binary_search(&field) {
            self.0.remove(n);
        };
        self
    }

    /// An iterater for a signature
    pub fn iter(&self) -> impl Iterator<Item = &ComponentId> {
        self.0.iter()
    }

    /// Finds all matching elements between two signatures
    pub fn each_shared(&self, other: &Self, mut func: impl FnMut(usize, usize)) {
        if self.0.is_empty() || other.0.is_empty() {
            return;
        }
        let [mut n, mut m] = [0; 2];
        while n < self.0.len() && self.0[n] < other.0[m] {
            n += 1;
        }

        if n == self.0.len() {
            return;
        }

        while m < other.0.len() && other.0[m] < self.0[n] {
            m += 1;
        }
        if m == other.0.len() {
            return;
        }

        while n < self.0.len() && m < other.0.len() {
            if self.0[n] == other.0[m] {
                func(n, m);
            }
            if self.0[n] < other.0[m] {
                n += 1;
            } else {
                m += 1;
            }
        }
    }
}

#[derive(Debug)]
pub struct Column {
    buffer: AVec<MaybeUninit<u8>, RuntimeAlign>,
    info: ComponentInfo,
}

impl Column {
    pub fn new(component_info: ComponentInfo) -> Self {
        Self {
            buffer: AVec::new(component_info.align),
            info: component_info,
        }
    }

    /// Gets a chunk of the column
    pub fn get_chunk(&self, RowIndex(row): RowIndex) -> &[MaybeUninit<u8>] {
        &self.buffer[row * self.info.size..][..self.info.size]
    }

    /// Gets a chunk of the column mutibly
    pub fn get_chunk_mut(&mut self, RowIndex(row): RowIndex) -> &mut [MaybeUninit<u8>] {
        &mut self.buffer[row * self.info.size..][..self.info.size]
    }

    /// Checks if the size is zero
    pub fn no_chunks(&self) -> usize {
        if self.info.size == 0 {
            0
        } else {
            self.buffer.len() / self.info.size
        }
    }

    /// Writes into the column
    pub unsafe fn write_into(&mut self, RowIndex(row): RowIndex, bytes: &[MaybeUninit<u8>]) {
        if self.info.size == 0 {
            return;
        }
        if row < self.no_chunks() {
            // SAFETY: chunk is writtten into
            unsafe { self.call_drop(RowIndex(row)) };
            self.buffer[row * self.info.size..].copy_from_slice(bytes);
        } else {
            self.buffer.extend_from_slice(bytes);
        }
    }

    // Must change length/overwrite bytes after call
    unsafe fn call_drop(&mut self, RowIndex(row): RowIndex) {
        let bytes = &mut self.buffer[row * self.info.size..][..self.info.size];
        debug_assert_eq!(bytes.len(), self.info.size);
        unsafe {
            (self.info.drop)(&mut self.buffer[row * self.info.size..][..self.info.size]);
        }
    }
}

impl Drop for Column {
    fn drop(&mut self) {
        if self.info.size == 0 {
            return;
        }
        for n in (0..self.buffer.len()).step_by(self.info.size) {
            unsafe { (self.info.drop)(&mut self.buffer[n..][..self.info.size]) }
        }
    }
}
