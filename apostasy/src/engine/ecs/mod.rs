use std::{
    arch,
    cell::{Cell, UnsafeCell},
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

use thread_local::ThreadLocal;

use crate::engine::ecs::{
    command::Command,
    component::COMPONENT_ENTRIES,
    core::Core,
    entity::{Entity, EntityLocation, EntityView},
    query::QueryBuilder,
};

pub mod archetype;
pub mod command;
pub mod component;
pub mod core;
pub mod entity;
pub mod query;

/// Wrapper for the Crust
pub struct World {
    pub crust: Arc<Crust>,
}

/// Container for the Mantle
pub struct Crust {
    pub mantle: UnsafeCell<Mantle>,
    pub flush_guard: AtomicUsize,
}

unsafe impl Send for Crust {}
unsafe impl Sync for Crust {}

/// Container for commands and the core
pub struct Mantle {
    pub core: Core,
    pub commands: ThreadLocal<Cell<Vec<Command>>>,
}

impl Mantle {
    /// Adds a command to the queue,
    /// rarely manually called
    pub fn queue_command(&self, command: Command) {
        let cell = self.commands.get_or(|| Cell::new(Vec::default()));
        let mut queue = cell.take();
        queue.push(command);
        cell.set(queue);
    }

    /// Applies every command currently in the queue
    /// rarely manually called
    pub fn apply_commands(&mut self) {
        for cell in self.commands.iter_mut() {
            for command in cell.get_mut().drain(..) {
                command.apply(&mut self.core);
            }
        }
    }

    /// Debugs all archetypes
    /// rarely manually called
    pub fn archetypes(&self) {
        for archetype in self.core.archetypes.slots.iter() {
            dbg!(archetype);
        }
    }
}

#[allow(clippy::redundant_pattern_matching)]
impl Crust {
    /// Opens access to the crust
    pub fn begin_access(flush_guard: &AtomicUsize) {
        if let Err(_) = flush_guard.fetch_update(Ordering::SeqCst, Ordering::SeqCst, |old| {
            (old < usize::MAX).then_some(old + 1)
        }) {
            panic!("Tried to read while structurally mutating");
        }
    }

    /// Closes access to the crust
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

    /// Opens access to the mantle
    /// use:
    /// ```rust
    ///
    ///     fn foo(){
    ///         let world = World::new();
    ///
    ///         world.crust.mantle(|mantle| {
    ///             ...
    ///         });
    ///     }
    ///
    /// ```
    pub fn mantle<R>(&self, func: impl FnOnce(&Mantle) -> R) -> R {
        Self::begin_access(&self.flush_guard);
        let ret = func(unsafe { self.mantle.get().as_ref().unwrap() });
        Self::end_access(&self.flush_guard);
        ret
    }

    /// Runs a flush, applies all commands
    pub fn flush(&self) {
        Self::begin_flush(&self.flush_guard);
        unsafe { self.mantle.get().as_mut().unwrap().apply_commands() };
        Self::end_flush(&self.flush_guard);
    }
}

#[allow(clippy::new_without_default)]
impl World {
    /// Creates a new World, use:
    /// ```rust
    ///     fn foo(){
    ///         let world = World::new();
    ///     }
    /// ```
    pub fn new() -> Self {
        let mut world = Self {
            crust: Arc::new(Crust {
                flush_guard: AtomicUsize::new(0),
                mantle: UnsafeCell::new(Mantle {
                    core: Core::new(),
                    commands: Default::default(),
                }),
            }),
        };

        for init in COMPONENT_ENTRIES {
            (init)(&mut world);
        }

        world.flush();

        world
    }

    /// Takes in an entity and returns it's EntityView, use:
    /// ```rust
    ///     fn foo(){
    ///         let world = World::new();
    ///         let entity = world.spawn();
    ///
    ///         let entity_view = world.entity(entity);
    ///     }
    /// ```
    pub fn entity(&self, entity: Entity) -> EntityView<'_> {
        let entity = self.get_entity(entity).unwrap();
        entity
    }

    /// Gets an option of EntityView
    pub fn get_entity(&self, entity: Entity) -> Option<EntityView<'_>> {
        self.crust.mantle(|mantle| {
            mantle
                .core
                .get_entity_location_locking(entity)
                .map(|_| EntityView {
                    entity,
                    world: self,
                })
        })
    }

    /// Spawns an entity, use:
    /// ```rust
    ///     fn foo(){
    ///         let world = World::new();
    ///
    ///         let entity = world.spawn();
    ///     }
    /// ```
    pub fn spawn(&self) -> EntityView<'_> {
        self.crust.mantle(|mantle| {
            let entity = mantle.core.create_unspawned_entity();
            mantle.queue_command(Command::spawn(entity));
            EntityView {
                entity,
                world: self,
            }
        })
    }

    pub fn entity_from_location(&self, entity_location: EntityLocation) -> EntityView<'_> {
        self.crust.mantle(|mantle| {
            let archetype = mantle
                .core
                .archetypes
                .get(entity_location.archetype)
                .unwrap();

            self.get_entity(
                archetype
                    .entity_index
                    .get(&entity_location)
                    .unwrap()
                    .to_owned(),
            )
            .unwrap()
        })
    }

    /// Despawns an entity, use:
    /// ```rust
    ///     fn foo(){
    ///         let world = World::new();
    ///
    ///         let entity = world.spawn();
    ///
    ///         world.despawn(entity.entity);
    ///     }
    /// ```
    pub fn despawn(&self, entity: Entity) {
        self.crust.mantle(|mantle| {
            mantle.queue_command(Command::despawn(entity));
        });
    }

    /// Runs all flush functions on the Crust, use:
    /// ```rust
    ///     #[derive(Component)]
    ///     struct A(f32);
    ///
    ///     fn foo(){
    ///         let world = World::new();
    ///
    ///         let entity = world.spawn().insert(A(0.0));
    ///
    ///         world.flush();
    ///     }
    pub fn flush(&self) {
        self.crust.flush();
    }

    /// Creates a new query, use:
    /// ```rust
    ///     #[derive(Component)]
    ///     struct A(f32);
    ///
    ///     fn foo(){
    ///         let world = World::new();
    ///
    ///         let entity = world.spawn().insert(A(0.0));
    ///
    ///         world
    ///             .query()
    ///             .with()
    ///             .include(A::id())
    ///             .build()
    ///             .run(|view: EntityView<'_>| {
    ///                 ...
    ///             });
    ///     }
    /// ```
    pub fn query(&self) -> QueryBuilder {
        QueryBuilder::new(World {
            crust: self.crust.clone(),
        })
    }
}
