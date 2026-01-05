use std::{
    cell::{Cell, UnsafeCell},
    collections::HashMap,
    mem::MaybeUninit,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

use parking_lot::{Mutex, RwLock};
use thread_local::ThreadLocal;

use crate::{
    engine::ecs::{
        archetype::{
            Archetype, ArchetypeEdge, ArchetypeId, Column, ColumnIndex, RowIndex, Signature,
        },
        command::Command,
        component::{COMPONENT_ENTRIES, Component, ComponentId, ComponentInfo, ComponentLocations},
        entity::{Entity, EntityLocation, EntityView},
    },
    utils::slotmap::SlotMap,
};

pub mod archetype;
pub mod command;
pub mod component;
pub mod entity;
pub mod query;

pub struct World {
    pub crust: Arc<Crust>,
}

pub struct Crust {
    pub mantle: UnsafeCell<Mantle>,
    pub flush_guard: AtomicUsize,
}

unsafe impl Send for Crust {}
unsafe impl Sync for Crust {}

pub struct Mantle {
    pub core: Core,
    pub commands: ThreadLocal<Cell<Vec<Command>>>,
}

/// The world of the ecs, it contains the archetypes and entities
pub struct Core {
    pub archetypes: SlotMap<ArchetypeId, Archetype>,
    pub entity_index: Mutex<SlotMap<Entity, EntityLocation>>,
    pub component_index: HashMap<ComponentId, ComponentLocations>,
    pub signature_index: HashMap<Signature, ArchetypeId>,
}

impl Mantle {
    pub fn queue_command(&self, command: Command) {
        let cell = self.commands.get_or(|| Cell::new(Vec::default()));
        let mut queue = cell.take();
        queue.push(command);
        cell.set(queue);
    }

    pub fn apply_commands(&mut self) {
        for cell in self.commands.iter_mut() {
            for command in cell.get_mut().drain(..) {
                command.apply(&mut self.core);
            }
        }
    }

    pub fn archetypes(&self) {
        for archetype in self.core.archetypes.slots.iter() {
            dbg!(archetype);
        }
    }
}

#[allow(clippy::redundant_pattern_matching)]
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
        unsafe { self.mantle.get().as_mut().unwrap().apply_commands() };
        Self::end_flush(&self.flush_guard);
    }
}

#[allow(clippy::new_without_default)]
impl World {
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

    pub fn entity(&self, entity: Entity) -> EntityView<'_> {
        self.get_entity(entity).unwrap()
    }

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

    /// Despawns an entity
    pub fn despawn(&self, entity: Entity) {
        self.crust.mantle(|mantle| {
            mantle.queue_command(Command::despawn(entity));
        });
    }

    pub fn flush(&self) {
        self.crust.flush();
    }
}

#[allow(clippy::new_without_default)]
impl Core {
    pub fn new() -> Self {
        // Create the slotmaps for entities and archetypes
        let mut archetypes = SlotMap::<ArchetypeId, Archetype>::default();
        let mut entity_index = SlotMap::<Entity, EntityLocation>::default();

        // Create the empty archetype and component info archetype
        let empty_archetype_id = archetypes.insert(Archetype::default());
        let component_info_archetype_id = archetypes.insert(Archetype::default());
        assert_eq!(empty_archetype_id, ArchetypeId::empty_archetype());
        assert_ne!(empty_archetype_id, component_info_archetype_id);

        if let Some(empty_archetype) = archetypes.get_mut(empty_archetype_id) {
            // Add all components as entities before init starts
            for n in 0..COMPONENT_ENTRIES.len() {
                let id = entity_index.insert(EntityLocation {
                    archetype: empty_archetype_id,
                    row: RowIndex(n),
                });
                empty_archetype.entities.push(id);
            }

            let component_edge = &mut empty_archetype
                .edges
                .entry(ComponentInfo::id().into())
                .or_default();
            component_edge.add = Some(component_info_archetype_id);
        }

        // Manually create ComponentInfo Archetype
        let component_info_signature = Signature::new(&[ComponentInfo::id().into()]);
        archetypes[component_info_archetype_id] = Archetype {
            signature: component_info_signature.clone(),
            entities: Default::default(),
            columns: vec![RwLock::new(Column::new(ComponentInfo::info()))],
            edges: HashMap::from([(
                ComponentInfo::id().into(),
                ArchetypeEdge {
                    remove: Some(empty_archetype_id),
                    add: None,
                },
            )]),
        };

        Core {
            archetypes,
            entity_index: Mutex::new(entity_index),
            component_index: HashMap::from([(
                ComponentInfo::id().into(),
                ComponentLocations(HashMap::from([(
                    component_info_archetype_id,
                    ColumnIndex(0),
                )])),
            )]),
            signature_index: HashMap::from([
                (Signature::default(), empty_archetype_id),
                (component_info_signature, component_info_archetype_id),
            ]),
        }
    }

    pub fn get_entity_location_locking(&self, entity: Entity) -> Option<EntityLocation> {
        let entity_index = self.entity_index.lock();
        entity_index.get(entity).copied()
    }

    pub fn create_unspawned_entity(&self) -> Entity {
        let mut entity_index = self.entity_index.lock();
        entity_index.insert(EntityLocation::uninitialized())
    }

    pub fn spawn_entity(&mut self, entity: Entity) -> Entity {
        let entity_index = self.entity_index.get_mut();
        let mut location = entity_index[entity];
        if location == EntityLocation::uninitialized() {
            let empty_archetype = &mut self.archetypes[ArchetypeId::empty_archetype()];
            location.row = RowIndex(empty_archetype.entities.len());
            empty_archetype.entities.push(entity);
            entity_index[entity] = location;
        }
        entity
    }

    pub fn despawn_entity(&mut self, entity: Entity) {
        let mut entity_index = self.entity_index.get_mut();
        let mut location = entity_index[entity];

        let archetype = &mut self.archetypes[location.archetype];
        let mut current_row = archetype.entities.get(location.row.0).unwrap();
        let mut final_row = archetype.entities.last().unwrap();

        // find entity's location
        // swap the current entity with the end entity
        // remove the end entity

        // Only swap if the current row isnt the last row
        if current_row != final_row {
            let s_row = final_row.clone();
            final_row = current_row;
            current_row = &s_row;
        }

        let final_entity = archetype.entities.last().unwrap().to_owned();
        let final_location = entity_index[final_entity];
        archetype.entities.remove(final_location.row.0);

        entity_index.remove(entity);
    }

    pub fn entity_location(&mut self, entity: Entity) -> Option<EntityLocation> {
        let entity_index = self.entity_index.get_mut();
        entity_index.get(entity).copied()
    }

    /// Returns the component info of a component
    fn get_component_info(
        entity_index: &SlotMap<Entity, EntityLocation>,
        component_index: &HashMap<ComponentId, ComponentLocations>,
        archetypes: &SlotMap<ArchetypeId, Archetype>,
        component: Entity,
    ) -> Option<ComponentInfo> {
        component_index
            .get(&ComponentInfo::id().into())
            .zip(entity_index.get_ignore_generation(component))
            .and_then(|(component_locations, component_location)| {
                let column = archetypes
                    .get(component_location.archetype)?
                    .columns
                    .get(**component_locations.get(&component_location.archetype)?)?
                    .read();
                let bytes = &column.get_chunk(component_location.row);
                let info = unsafe { std::ptr::read(bytes.as_ptr() as *const ComponentInfo) };
                Some(info)
            })
    }

    /// Get data of a component
    pub fn component_info(&mut self, component: Entity) -> Option<ComponentInfo> {
        let entity_index = self.entity_index.get_mut();
        let component_index = &self.component_index;
        let archetypes = &self.archetypes;
        Self::get_component_info(entity_index, component_index, archetypes, component)
    }

    pub fn create_archetype(&mut self, signature: Signature) -> ArchetypeId {
        if let Some(id) = self.signature_index.get(&signature) {
            *id
        } else {
            let mut new_archetype = Archetype {
                signature: signature.clone(),
                entities: Default::default(),
                columns: Default::default(),
                edges: Default::default(),
            };

            // Create columns
            for component in signature.iter() {
                println!("{}", component.as_entity().is_some());

                let info = self.component_info(component.as_entity().unwrap()).unwrap();
                new_archetype.columns.push(RwLock::new(Column::new(info)));
            }

            let id = self.archetypes.insert(new_archetype);
            self.signature_index.insert(signature.clone(), id);

            for (n, component) in signature.iter().enumerate() {
                self.component_index
                    .entry(*component)
                    .or_default()
                    .insert(id, ColumnIndex(n));
            }

            // do edge connections
            self.connect_edges(signature, id);

            id
        }
    }

    fn connect_edges(&mut self, signature: Signature, id: ArchetypeId) {
        for field in signature.iter() {
            let without_field = signature.clone().without(*field);
            let Some(other) = self.signature_index.get(&without_field).copied() else {
                continue;
            };

            // Connect this to other
            self.archetypes[id].edges.entry(*field).or_default().remove = Some(other);

            // Connect other to this
            self.archetypes[other].edges.entry(*field).or_default().add = Some(id);
        }
    }

    pub unsafe fn insert_component_bytes(
        &mut self,
        info: ComponentInfo,
        bytes: &[MaybeUninit<u8>],
        entity: Entity,
    ) -> EntityLocation {
        let Some(current_location) = self.entity_location(entity) else {
            panic!("Entity does not exist");
        };

        let current_archetype = &self.archetypes[current_location.archetype];
        let entity = current_archetype.entities[*current_location.row];

        // Find destination archetype
        let destination = if current_archetype.signature.contains(info.id.into()) {
            current_location.archetype
        } else if let Some(edge) = current_archetype
            .edges
            .get(&info.id.into())
            .and_then(|edge| edge.add)
        {
            edge
        } else {
            self.create_archetype(current_archetype.signature.clone().with(info.id.into()))
        };

        // SAFETY: New chunk is immediately created for entity
        // Move entity
        unsafe { self.move_entity(current_location, destination) };

        // SAFETY:
        //  - component info should match the columns
        //  - chunk for the row is moved if a new archetype is created
        //  - write_info will call drop on old component value if not move to a new archetype
        let updated_location = self.entity_location(entity).unwrap();
        unsafe {
            let column = self.component_index[&info.id.into()][&updated_location.archetype];

            self.archetypes[destination] //
                .columns[*column]
                .get_mut()
                .write_into(updated_location.row, bytes);
        }

        updated_location
    }

    unsafe fn move_entity(
        &mut self,
        old_location: EntityLocation,
        destination_id: ArchetypeId,
    ) -> EntityLocation {
        if old_location.archetype == destination_id {
            return old_location;
        }
        let entity_index = self.entity_index.get_mut();
        let [old_archetype, new_archetype] = self
            .archetypes
            .disjoint([old_location.archetype, destination_id])
            .unwrap();

        let entity = old_archetype.entities.swap_remove(*old_location.row);
        new_archetype.entities.push(entity);

        old_archetype
            .signature
            .each_shared(&new_archetype.signature, |n, m| {
                let old_column = old_archetype.columns[n].get_mut();
                let new_column = new_archetype.columns[m].get_mut();
                old_column.move_into(new_column, old_location.row);
            });

        // Update entity locations
        let updated_location = EntityLocation {
            archetype: destination_id,
            row: RowIndex(new_archetype.entities.len() - 1),
        };
        entity_index[entity] = updated_location;
        if *old_location.row < old_archetype.entities.len() {
            entity_index[old_archetype.entities[*old_location.row]].row = old_location.row;
        }

        for column in old_archetype.columns.iter() {
            column.write().shrink_to_fit(old_archetype.entities.len());
        }

        updated_location
    }
    pub fn remove_component(&mut self, component: ComponentId, entity: Entity) -> EntityLocation {
        // Look for the entity
        let Some(current_location) = self.entity_location(entity) else {
            panic!("Entity does not exist");
        };
        let current_archetype = &self.archetypes[current_location.archetype];

        // Find the new destination
        let destination = if let Some(edge) = current_archetype //
            .edges
            .get(&component)
            .and_then(|edge| edge.remove)
        {
            edge
        } else {
            self.create_archetype(current_archetype.signature.clone().without(component))
        };

        // SAFETY: Should only ever drop components
        unsafe { self.move_entity(current_location, destination) }
    }
}
