use std::{
    ops::{Deref, DerefMut},
    sync::atomic::AtomicUsize,
};

use derive_more::From;
use parking_lot::{MappedRwLockReadGuard, MappedRwLockWriteGuard};

use crate::{
    engine::ecs::{
        Crust, World,
        archetype::{ArchetypeId, RowIndex},
        command::Command,
        component::{Component, ComponentId},
    },
    utils::slotmap::Key,
};

/// The location of an entity in an archetype
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

/// They key for an entity
#[derive(Clone, Copy, Debug, From, PartialEq, Eq)]
pub struct Entity(Key);

impl From<Entity> for Key {
    fn from(value: Entity) -> Self {
        value.0
    }
}

impl Entity {
    /// # Safety
    /// Should never be called manually
    pub unsafe fn from_offset(val: u32) -> Self {
        Self(Key {
            index: val,
            generation: 1,
        })
    }

    pub fn raw(self) -> u64 {
        self.0.raw()
    }

    pub fn from_raw(val: u64) -> Self {
        Self(Key::from_raw(val))
    }
}

#[derive(Clone, Copy)]
pub struct EntityView<'a> {
    pub entity: Entity,
    pub world: &'a World,
}

impl EntityView<'_> {
    pub fn id(&self) -> Entity {
        self.entity
    }
    pub fn insert<C: Component>(self, component: C) -> Self {
        self.world.crust.mantle(|mantle| {
            mantle.queue_command(Command::insert(component, self.entity));
        });
        self
    }

    pub fn remove<C: Into<ComponentId>>(self, component: C) -> Self {
        self.world.crust.mantle(|mantle| {
            mantle.queue_command(Command::remove(component.into(), self.entity));
        });
        self
    }

    /// Get a component on an entity immutably
    pub fn get<T: Component>(&self) -> Option<ColumnReadGuard<'_, T>> {
        // Open access to the crust
        Crust::begin_access(&self.world.crust.flush_guard);

        let core = unsafe { &self.world.crust.mantle.get().as_ref().unwrap().core };
        let location = core.get_entity_location_locking(self.entity).unwrap();
        let out = core.get_bytes(T::id().into(), location).map(|bytes| {
            ColumnReadGuard::new(
                MappedRwLockReadGuard::map(bytes, |bytes| {
                    unsafe { (bytes.as_ptr() as *const T).as_ref() }.unwrap()
                }),
                &self.world.crust.flush_guard,
            )
        });

        // Close access to the crust
        Crust::end_access(&self.world.crust.flush_guard);

        out
    }

    /// Get a component on an entity mutably
    pub fn get_mut<T: Component>(&self) -> Option<ColumnWriteGuard<'_, T>> {
        // Open access to the crust
        Crust::begin_access(&self.world.crust.flush_guard);

        let core = unsafe { &self.world.crust.mantle.get().as_ref().unwrap().core };
        let location = core.get_entity_location_locking(self.entity).unwrap();
        let out = core.get_bytes_mut(T::id().into(), location).map(|bytes| {
            ColumnWriteGuard::new(
                MappedRwLockWriteGuard::map(bytes, |bytes| {
                    unsafe { (bytes.as_ptr() as *mut T).as_mut() }.unwrap()
                }),
                &self.world.crust.flush_guard,
            )
        });

        // Close access to the crust
        Crust::end_access(&self.world.crust.flush_guard);
        out
    }
}

pub struct ColumnReadGuard<'a, T> {
    mapped_guard: MappedRwLockReadGuard<'a, T>,
    flush_guard: *const AtomicUsize,
}

impl<'a, T> ColumnReadGuard<'a, T> {
    pub fn new(mapped_guard: MappedRwLockReadGuard<'a, T>, flush_guard: &AtomicUsize) -> Self {
        Crust::begin_access(flush_guard);
        Self {
            mapped_guard,
            flush_guard,
        }
    }
}
impl<T> Deref for ColumnReadGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.mapped_guard
    }
}

impl<T> Drop for ColumnReadGuard<'_, T> {
    fn drop(&mut self) {
        // SAFETY: Always safe because atomic
        Crust::end_access(unsafe { self.flush_guard.as_ref().unwrap() });
    }
}

pub struct ColumnWriteGuard<'a, T> {
    mapped_guard: MappedRwLockWriteGuard<'a, T>,
    flush_guard: *const AtomicUsize,
}

impl<'a, T> ColumnWriteGuard<'a, T> {
    pub fn new(mapped_guard: MappedRwLockWriteGuard<'a, T>, flush_guard: &AtomicUsize) -> Self {
        Crust::begin_access(flush_guard);
        Self {
            mapped_guard,
            flush_guard,
        }
    }
}

impl<T> Deref for ColumnWriteGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.mapped_guard
    }
}
impl<T> DerefMut for ColumnWriteGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.mapped_guard
    }
}
impl<T> Drop for ColumnWriteGuard<'_, T> {
    fn drop(&mut self) {
        // SAFETY: Always safe because atomic
        Crust::end_access(unsafe { self.flush_guard.as_ref().unwrap() });
    }
}
