use std::{
    cell::{Cell, UnsafeCell},
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

use thread_local::ThreadLocal;

use crate::engine::ecs::{
    entity::Entity,
    world::{commands::Command, core::Core},
};

pub mod archetype;
pub mod commands;
pub mod core;

/// Wrapper for Crust
pub struct World {
    pub crust: Arc<Crust>,
}

/// Wrapper for Crust
pub struct Crust {
    pub mantle: UnsafeCell<Mantle>,
    pub flush_guard: AtomicUsize,
}

/// Wrapper for Core
pub struct Mantle {
    pub core: Core,
    pub commands: ThreadLocal<Cell<Vec<Command>>>,
}
impl Mantle {
    pub fn enqueue(&self, command: Command) {
        let cell = self.commands.get_or(|| Cell::new(Vec::default()));
        let mut queue = cell.take();
        queue.push(command);
        cell.set(queue);
    }

    pub fn flush(&mut self) {
        for cell in self.commands.iter_mut() {
            for command in cell.get_mut().drain(..) {
                command.apply(&mut self.core);
            }
        }
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

    pub fn flush(&self) {
        Self::begin_flush(&self.flush_guard);
        unsafe { self.mantle.get().as_mut().unwrap().flush() };
        Self::end_flush(&self.flush_guard);
    }
}

impl World {
    pub fn new() -> Self {
        Self {
            crust: Arc::new(Crust {
                flush_guard: AtomicUsize::new(0),
                mantle: UnsafeCell::new(Mantle {
                    core: Core::new(),
                    commands: Default::default(),
                }),
            }),
        }
    }

    // pub fn entity(&self, entity: Entity) -> View<'_> {
    //     self.get_entity(entity).unwrap()
    // }
    //
    pub fn spawn(&self) {
        self.crust.mantle(|mantle| {
            let entity = mantle.core.create_uninitalized_entity_location();
            mantle.enqueue(Command::spawn(entity));
        });
    }
}
