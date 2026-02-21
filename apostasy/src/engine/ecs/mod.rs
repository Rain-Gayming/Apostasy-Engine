use crate::log;
use std::fmt;
use std::{
    cell::{Cell, UnsafeCell},
    mem::MaybeUninit,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

use parking_lot::RwLock;
use thread_local::ThreadLocal;

use crate::engine::editor::EditorStorage;
use crate::engine::{
    ecs::{
        archetype::{ArchetypeDebug, RowIndex},
        command::Command,
        component::{COMPONENT_ENTRIES, Component, ComponentInfo},
        core::Core,
        entity::{Entity, EntityLocation, EntityView},
        query::QueryBuilder,
        resource::{Resource, ResourceMap, ResourcesGetter},
        resources::{frame_counter::FPSCounter, input_manager::InputManager},
        system::{FixedUpdateSystem, LateUpdateSystem, StartSystem, UpdateSystem},
    },
    rendering::{models::model::ModelLoader, rendering_context::RenderingContext},
    voxels::{chunk_loader::ChunkStorage, voxel_registry::VoxelRegistry},
    windowing::cursor_manager::CursorManager,
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
    /// Default package
    /// Includes:
    /// - InputManager
    /// - CursorManager
    /// - ModelLoader
    Default,
    /// Debug package
    /// Includes:
    /// - FPSCounter
    Debug,
    /// Editor package
    /// Includes:
    /// - EditorStorage
    Editor,
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
                self.insert_resource(ChunkStorage::default());
                self.insert_resource(VoxelRegistry::default());
            }
            Package::Default => {
                self.insert_resource(InputManager::default());
                self.insert_resource(CursorManager::default());
                self.insert_resource(ModelLoader::default());
            }
            Package::Debug => {
                self.insert_resource(FPSCounter::default());
            }
            Package::Editor => {
                self.insert_resource(EditorStorage::default());
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

    pub fn entity_from_location(&self, entity_location: EntityLocation) -> Option<EntityView<'_>> {
        self.crust.mantle(|mantle| {
            let archetype = mantle
                .core
                .archetypes
                .get(entity_location.archetype)
                .unwrap();

            if archetype.entity_index.get(&entity_location).is_none() {
                return None;
            }

            self.get_entity(
                archetype
                    .entity_index
                    .get(&entity_location)
                    .unwrap()
                    .to_owned(),
            )
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
    ///             log!("Delta: {}", time.value);
    ///         });
    ///     }
    /// ```
    pub fn with_resource<T: Resource, F, R>(&self, func: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        self.crust.mantle(|mantle| {
            if mantle.resources.is_locked() {
                log!("Resources is currently locked");
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
    ///             log!("Delta: {}", time.value);
    ///         });
    ///     }
    /// ```
    pub fn with_resource_mut<T: Resource, F, R>(&self, func: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        self.crust.mantle(|mantle| {
            if mantle.resources.is_locked() {
                log!("Resources is currently locked");
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
    ///             log!("Delta: {}", time.value);
    ///         });
    ///     }
    /// ```
    pub fn with_resources<T: ResourcesGetter, R>(
        &self,
        func: impl FnOnce(T::Output<'_>) -> R,
    ) -> R {
        self.crust.mantle(|mantle| {
            if mantle.resources.is_locked() {
                log!("Resources is currently locked");
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

    pub fn get_all_entities(&self) -> Vec<Entity> {
        self.crust.mantle(|mantle| {
            let mut entities: Vec<Entity> = Vec::new();

            for archetype in mantle.core.archetypes.slots.iter() {
                if archetype
                    .data
                    .as_ref()
                    .unwrap()
                    .entities
                    .contains(&ComponentInfo::id())
                {
                    continue;
                }
                if let Some(data) = &archetype.data {
                    for entity in data.entities.iter() {
                        entities.push(*entity);
                    }
                }
            }

            entities
        })
    }

    /// Returns an entities component information as a string from a location
    /// primary used for debugging
    pub fn get_component_info(&self, entity_location: EntityLocation) -> String {
        let mut string = String::new();
        let entity = self.entity_from_location(entity_location);
        self.crust.mantle(|mantle| {
            let core = &mantle.core;
            let archetype = core.archetypes.get(entity_location.archetype).unwrap();
            if archetype.entities.contains(&ComponentInfo::id()) {
                return;
            }

            let component_infos: Vec<ComponentInfo> = archetype
                .signature
                .iter()
                .filter_map(|component_id| component_id.as_entity())
                .filter_map(|entity| {
                    let component_info_locations = core
                        .component_index
                        .get(&ComponentInfo::id().into())
                        .unwrap();

                    let entity_index = core.entity_index.lock();
                    let comp_location = entity_index.get_ignore_generation(entity).copied()?;
                    drop(entity_index);

                    let col_index = *component_info_locations.get(&comp_location.archetype)?;

                    let archetype = core.archetypes.get(comp_location.archetype).unwrap();
                    let column = archetype.columns.get(*col_index)?.read();
                    let bytes = column.get_chunk(comp_location.row);

                    Some(unsafe { std::ptr::read(bytes.as_ptr() as *const ComponentInfo) })
                })
                .collect();

            struct FmtWrapper<'a> {
                bytes: &'a [MaybeUninit<u8>],
                fmt_fn: fn(&[MaybeUninit<u8>], &mut fmt::Formatter) -> fmt::Result,
            }
            impl fmt::Debug for FmtWrapper<'_> {
                fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    (self.fmt_fn)(self.bytes, f)
                }
            }

            let components: Vec<String> = archetype
                .columns
                .iter()
                .zip(component_infos.iter())
                .map(|(col, info)| {
                    let col = col.read();
                    let bytes = col.get_chunk(RowIndex(entity_location.row.0));
                    let value = match info.fmt {
                        Some(fmt_fn) => format!("{:?}", FmtWrapper { bytes, fmt_fn }),
                        None => "(no Debug impl)".to_string(),
                    };
                    format!("{}: {}", info.name, value)
                })
                .collect();

            if entity.is_some() {
                string = format!(
                    "{:?} => {{ {} }}",
                    entity.unwrap().entity,
                    components.join(", ")
                );
            }
        });
        string
    }

    /// Returns a string of all archetypes and their components
    pub fn debug_archetypes(&self) -> String {
        self.crust.mantle(|mantle| {
            let mut string = String::new();
            for archetype in mantle.core.archetypes.slots.iter().skip(2) {
                if let Some(data) = &archetype.data {
                    string.push_str(&format!(
                        "{:#?}\n",
                        ArchetypeDebug {
                            archetype: data,
                            core: &mantle.core,
                        }
                    ));
                }
            }
            string
        })
    }

    pub fn get_component_info_by_name(&self, name: &str) -> Option<ComponentInfo> {
        self.crust.mantle(|mantle| {
            let core = &mantle.core;
            let component_info_locations = core.component_index.get(&ComponentInfo::id().into())?;

            // There's only one archetype holding ComponentInfos
            let (archetype_id, col_index) = component_info_locations.iter().next()?;
            let archetype = core.archetypes.get(*archetype_id)?;
            let column = archetype.columns.get(**col_index)?.read();

            // Linear scan through all ComponentInfos looking for a name match
            let num_entities = archetype.entities.len();
            for row in 0..num_entities {
                let bytes = column.get_chunk(RowIndex(row));
                let info = unsafe { std::ptr::read(bytes.as_ptr() as *const ComponentInfo) };
                if info.name.to_lowercase() == name.to_lowercase()
                    || info
                        .name
                        .to_lowercase()
                        .ends_with(&format!("::{}", name.to_lowercase()))
                {
                    return Some(info);
                }
            }
            None
        })
    }

    pub fn add_default_component_by_name(&self, entity: Entity, name: &str) -> bool {
        let Some(info) = self.get_component_info_by_name(name.to_lowercase().as_str()) else {
            return false;
        };
        let Some(default_fn) = info.default else {
            log!("Component '{}' has no Default impl", name);
            return false;
        };

        let default_bytes: Box<[MaybeUninit<u8>]> = default_fn().into();
        self.crust.mantle(|mantle| unsafe {
            mantle.queue_command(Command::insert_bytes(info, default_bytes, entity));
        });
        true
    }
}

/// Returns a string of all entities and their components
/// primary used for debugging and the editor
pub fn entity_components_to_string(world: &World, entity: Entity) -> String {
    let mut result = String::new();

    world.crust.mantle(|mantle| {
        let core = &mantle.core;

        // Get entity location
        let location = match core.get_entity_location_locking(entity) {
            Some(loc) => loc,
            None => {
                result = "Entity not found".to_string();
                return;
            }
        };

        // Get the archetype
        let archetype = match core.archetypes.get(location.archetype) {
            Some(a) => a,
            None => {
                result = "Archetype not found".to_string();
                return;
            }
        };

        result.push_str(&format!("Entity {:?}:\n", entity));

        // Iterate over each component in the signature
        for (col_idx, component_id) in archetype.signature.iter().enumerate() {
            // Try to get ComponentInfo for this component
            let component_entity = match component_id.as_entity() {
                Some(e) => e,
                None => continue,
            };

            // Look up component info from the component_info archetype
            let info = core
                .component_index
                .get(&ComponentInfo::id().into())
                .and_then(|locations| {
                    let comp_location = core
                        .entity_index
                        .lock()
                        .get_ignore_generation(component_entity)
                        .copied()?;
                    let col_index = *locations.get(&comp_location.archetype)?;
                    let column = archetype.columns.get(*col_index)?.read();
                    // Read ComponentInfo bytes
                    let bytes = column.get_chunk(comp_location.row);
                    Some(unsafe { std::ptr::read(bytes.as_ptr() as *const ComponentInfo) })
                });

            match info {
                Some(comp_info) => {
                    let column = archetype.columns[col_idx].read();
                    let bytes = column.get_chunk(location.row);
                    result.push_str(&format!(
                        "  Component '{}' (size: {} bytes, align: {}): {:?}\n",
                        comp_info.name, comp_info.size, comp_info.align, bytes
                    ));
                }
                None => {
                    result.push_str(&format!("  ComponentId {:?}: (no info)\n", component_id));
                }
            }
        }
    });

    result
}
