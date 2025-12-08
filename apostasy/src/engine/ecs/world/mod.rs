use std::{
    cell::{Cell, UnsafeCell},
    process::Command,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

use thread_local::ThreadLocal;

use crate::engine::ecs::{
    entity::{Entity, View},
    world::core::Core,
};

pub mod archetype;
pub mod commands;
pub mod core;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct UintID(u64);

/// Wrapper for Crust
pub struct World {
    pub crust: Arc<Crust>,
}

/// Wrapper for Mantle
pub struct Crust {
    pub mantle: UnsafeCell<Mantle>,
    pub flush_guard: AtomicUsize,
}

unsafe impl Send for Crust {}
unsafe impl Sync for Crust {}

/// Wrapper for Core
pub struct Mantle {
    pub core: Core,
    pub commands: ThreadLocal<Cell<Vec<Command>>>,
}

impl Mantle {
    /// Adds a command to the queue
    pub fn queue_command(&self, command: Command) {
        let cell = self.commands.get_or(|| Cell::new(Vec::default()));
        let mut queue = cell.take();
        queue.push(command);
        cell.set(queue);
    }
}

impl Crust {
    pub fn begin_access(flush_guard: &AtomicUsize) {
        if let Err(_) = flush_guard.fetch_update(Ordering::SeqCst, Ordering::SeqCst, |old| {
            (old < usize::MAX).then_some(old + 1)
        }) {
            panic!("Tried to read while structurally mutating");
        }
    }

    pub fn end_access(flush_guard: &AtomicUsize) {
        if let Err(_) = flush_guard.fetch_update(Ordering::SeqCst, Ordering::SeqCst, |old| {
            (0 < old && old < usize::MAX).then_some(old - 1)
        }) {
            panic!("No read to end");
        }
    }

    pub fn begin_flush(flush_guard: &AtomicUsize) {
        if let Err(_) = flush_guard.fetch_update(Ordering::SeqCst, Ordering::SeqCst, |old| {
            (0 == old).then_some(usize::MAX)
        }) {
            panic!("Tried to structurally mutate while reading");
        }
    }

    pub fn end_flush(flush_guard: &AtomicUsize) {
        if let Err(_) = flush_guard.fetch_update(Ordering::SeqCst, Ordering::SeqCst, |old| {
            (old == usize::MAX).then_some(0)
        }) {
            panic!("No write to end");
        }
    }

    pub fn mantle<R>(&self, func: impl FnOnce(&Mantle) -> R) -> R {
        Self::begin_access(&self.flush_guard);
        let ret = func(unsafe { self.mantle.get().as_ref().unwrap() });
        Self::end_access(&self.flush_guard);
        ret
    }
}

impl World {
    pub fn get_entity(&self, entity: Entity) -> Option<View<'_>> {
        None
    }

    pub fn entity(&self, entity: Entity) -> View<'_> {
        self.get_entity(entity).unwrap()
    }
}
