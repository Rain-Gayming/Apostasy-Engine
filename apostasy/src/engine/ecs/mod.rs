use std::{
    cell::{Cell, UnsafeCell},
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

use parking_lot::RwLock;
use thread_local::ThreadLocal;

use crate::engine::{
    ecs::{
        command::Command,
        component::COMPONENT_ENTRIES,
        core::Core,
        entity::{Entity, EntityLocation, EntityView},
        query::QueryBuilder,
        resource::{Resource, ResourceMap, ResourcesGetter},
        system::{FixedUpdateSystem, LateUpdateSystem, StartSystem, UpdateSystem},
    },
    rendering::rendering_context::RenderingContext,
    voxels::{chunk_loader::ChunkStorage, voxel_registry::VoxelRegistry},
};

pub mod archetype;
pub mod command;
pub mod component;
pub mod components;
pub mod core;
pub mod entity;
pub mod query;
pub mod resource;
pub mod resources;
pub mod system;

/// A package that can be implimented into the world
#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub enum Package {
    /// Used for voxels and voxel rendering
    /// Adds the following resources:
    /// - ChunkStorage
    /// - VoxelRegistry
    /// Requires you to pass the VoxelRegistry location with:
    /// ```rust
    ///      world.with_resource_mut::<VoxelRegistry, _>(|registry| {
    ///         registry.load_from_directory("path").unwrap();j
    ///     });
    ///
    /// ```
    /// path is recomended to be "res/assets/voxels/"
    Voxels,
}

/// Wrapper for the Crust
pub struct World {
    pub crust: Arc<Crust>,
    pub rendering_context: Arc<RenderingContext>,
    pub packages: Vec<Package>,
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
    pub resources: RwLock<ResourceMap>,
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
    pub fn new(rendering_context: Arc<RenderingContext>) -> Self {
        let mut world = Self {
            crust: Arc::new(Crust {
                flush_guard: AtomicUsize::new(0),
                mantle: UnsafeCell::new(Mantle {
                    core: Core::new(),
                    commands: Default::default(),
                    resources: Default::default(),
                }),
            }),
            rendering_context,
            packages: Vec::new(),
        };

        for init in COMPONENT_ENTRIES {
            (init)(&mut world);
        }

        world.flush();

        world
    }

    /// Adds a package to the world and it's required resources, use:
    /// ```rust
    ///     fn foo(){
    ///         let world = World::new();
    ///
    ///         world.with_package(Package::Voxels);
    ///     }
    /// ```
    pub fn with_package(&mut self, package: Package) {
        self.packages.push(package);
        match package {
            Package::Voxels => {
                self.insert_resource::<ChunkStorage>(ChunkStorage::default());
                self.insert_resource::<VoxelRegistry>(VoxelRegistry::default());
            }
        }
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
        self.get_entity(entity).unwrap()
    }

    /// Gets an option of EntityView, use:
    /// ```rust
    ///     fn foo(){
    ///         let world = World::new();
    ///
    ///         let entity = world.spawn();
    ///
    ///         let entity_view = world.get_entity(entity);
    ///     }
    /// ```
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

    /// Runs every function with the #[update] attribute, use:
    /// ```rust
    ///     
    ///     #[update]
    ///     fn foo(world: &mut World) {
    ///         world.entity(entity).insert(A(0.0));
    ///     }
    /// ```
    pub fn update(&mut self) {
        let mut systems: Vec<&UpdateSystem> = inventory::iter::<UpdateSystem>.into_iter().collect();
        systems.sort_by_key(|s| s.priority);
        systems.reverse();

        for system in systems {
            (system.func)(self);
            self.flush();
        }
    }

    /// Runs every function with the #[fixed_update] attribute, use:
    /// ```rust
    ///     
    ///     #[fixed_update]
    ///     fn foo(world: &mut World) {
    ///         world.entity(entity).insert(A(0.0));
    ///     }
    /// ```
    pub fn fixed_update(&mut self, tick: f32) {
        let mut systems: Vec<&FixedUpdateSystem> =
            inventory::iter::<FixedUpdateSystem>.into_iter().collect();
        systems.sort_by_key(|s| s.priority);
        systems.reverse();
        for system in systems {
            (system.func)(self, tick);
            self.flush();
        }
    }

    /// Runs every function with the #[late_update] attribute, use:
    /// ```rust
    ///     
    ///     #[late_update]
    ///     fn foo(world: &mut World) {
    ///         world.entity(entity).insert(A(0.0));
    ///     }
    /// ```
    pub fn late_update(&mut self) {
        let mut systems: Vec<&LateUpdateSystem> =
            inventory::iter::<LateUpdateSystem>.into_iter().collect();
        systems.sort_by_key(|s| s.priority);
        systems.reverse();
        for system in systems {
            (system.func)(self);
            self.flush();
        }
    }

    /// Runs every function with the #[start] attribute, use:
    /// ```rust
    ///     
    ///     #[start]
    ///     fn foo(world: &mut World) {
    ///         world.entity(entity).insert(A(0.0));
    ///     }
    /// ```
    pub fn start(&mut self) {
        let mut systems: Vec<&StartSystem> = inventory::iter::<StartSystem>.into_iter().collect();
        systems.sort_by_key(|s| s.priority);
        systems.reverse();

        for system in systems {
            (system.func)(self);
            self.flush();
        }
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

    /// Inserts a resource, use:
    /// ```rust
    ///     #[derive(Resource)]
    ///     struct MyResource {
    ///         pub value: i32,
    ///     }
    ///
    ///     fn foo(){
    ///         let world = World::new();
    ///
    ///         world.insert_resource::<MyResource>(MyResource { value: 42 });
    ///     }
    /// ```
    pub fn insert_resource<T: Resource>(&self, resource: T) {
        self.crust.mantle(|mantle| {
            if mantle.resources.read().get::<T>().is_some() {
                panic!("Resource ({}) already exists", T::name());
            }
            mantle.resources.write().insert(resource);
        });
    }

    /// Gets a resource, use:
    /// ```rust
    ///     #[derive(Resource)]
    ///     struct MyResource {
    ///         pub value: i32,
    ///     }
    ///
    ///     fn foo(){
    ///         let world = World::new();
    ///
    ///         world.insert_resource::<MyResource>(MyResource { value: 42 });
    ///         world.with_resource>(|time: MyResource| {
    ///             println!("Delta: {}", time.value);
    ///         });
    ///     }
    /// ```
    pub fn with_resource<T: Resource, F, R>(&self, func: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        self.crust.mantle(|mantle| {
            if mantle.resources.is_locked() {
                println!("Resources is currently locked");
            }
            let resources = mantle.resources.read();
            if let Some(resource) = resources.get::<T>() {
                func(resource)
            } else {
                panic!("resource ({}) not found", T::name());
            }
        })
    }

    /// Gets a resource mutably, use:
    /// ```rust
    ///     #[derive(Resource)]
    ///     struct MyResource {
    ///         pub value: i32,
    ///     }
    ///
    ///     fn foo(){
    ///         let world = World::new();
    ///
    ///         world.insert_resource::<MyResource>(MyResource { value: 42 });
    ///         world.with_resource_mut(|time: MyResource| {
    ///             time.value += 1;
    ///             println!("Delta: {}", time.value);
    ///         });
    ///     }
    /// ```
    pub fn with_resource_mut<T: Resource, F, R>(&self, func: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        self.crust.mantle(|mantle| {
            if mantle.resources.is_locked() {
                println!("Resources is currently locked");
            }
            let mut resources = mantle.resources.write();
            if let Some(resource) = resources.get_mut::<T>() {
                func(resource)
            } else {
                panic!("resource ({}) not found", T::name());
            }
        })
    }
    /// Runs a function with a resource, use:
    /// ```rust
    ///     #[derive(Resource)]
    ///     struct MyResource {
    ///         pub value: i32,
    ///     }
    ///
    ///     fn foo(){
    ///         world.with_resources::<MyResource, _>(|time| {
    ///             println!("Delta: {}", time.value);
    ///         });
    ///     }
    /// ```
    pub fn with_resources<T: ResourcesGetter, R>(
        &self,
        func: impl FnOnce(T::Output<'_>) -> R,
    ) -> R {
        self.crust.mantle(|mantle| {
            if mantle.resources.is_locked() {
                println!("Resources is currently locked");
            }
            let mut resources = mantle.resources.write();
            func(T::get(&mut resources))
        })
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
            rendering_context: self.rendering_context.clone(),
            packages: Vec::new(),
        })
    }
}
