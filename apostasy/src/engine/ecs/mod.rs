use std::{cell::Cell, collections::HashMap, mem::MaybeUninit};

use parking_lot::Mutex;
use thread_local::ThreadLocal;

use crate::{
    engine::ecs::{
        archetype::{Archetype, ArchetypeId, ColumnIndex, RowIndex, Signature},
        command::Command,
        component::{Component, ComponentId, ComponentInfo, ComponentLocations},
        entity::{Entity, EntityLocation, EntityView},
    },
    utils::slotmap::SlotMap,
};

pub mod archetype;
pub mod command;
pub mod component;
pub mod entity;

/// The world of the ecs, it contains the archetypes and entities
pub struct World {
    pub archetypes: SlotMap<ArchetypeId, Archetype>,
    pub entity_index: Mutex<SlotMap<Entity, EntityLocation>>,
    pub component_index: HashMap<ComponentId, ComponentLocations>,
    pub signature_index: HashMap<Signature, ArchetypeId>,
    pub commands: ThreadLocal<Cell<Vec<Command>>>,
}

#[allow(clippy::new_without_default)]
impl World {
    pub fn new() -> Self {
        let mut archetypes = SlotMap::<ArchetypeId, Archetype>::default();
        let entity_index = SlotMap::<Entity, EntityLocation>::default();

        let empty_archetype = Archetype {
            signature: Signature::new(&[]),
            entities: Vec::new(),
            columns: Vec::new(),
        };

        archetypes.insert(empty_archetype);

        World {
            archetypes,
            entity_index: Mutex::new(entity_index),
            component_index: HashMap::new(),
            signature_index: HashMap::new(),
            commands: ThreadLocal::new(),
        }
    }

    pub fn entity(&self, entity: Entity) -> EntityView<'_> {
        self.get_entity(entity).unwrap()
    }

    pub fn queue_command(&self, command: Command) {
        let cell = self.commands.get_or(|| Cell::new(Vec::default()));
        let mut queue = cell.take();
        queue.push(command);
        cell.set(queue);
    }

    pub fn get_entity(&self, entity: Entity) -> Option<EntityView<'_>> {
        self.get_entity_location_locking(entity)
            .map(|_| EntityView {
                entity,
                world: self,
            })
    }

    pub fn get_entity_location_locking(&self, entity: Entity) -> Option<EntityLocation> {
        let entity_index = self.entity_index.lock();
        entity_index.get(entity).copied()
    }

    pub fn spawn(&mut self) {
        let entity = self.create_unspawned_entity();
        self.spawn_entity(entity);
    }

    pub fn create_unspawned_entity(&mut self) -> Entity {
        let mut entity_index = self.entity_index.lock();
        entity_index.insert(EntityLocation::uninitialized())
    }

    pub fn spawn_entity(&mut self, entity: Entity) -> EntityLocation {
        let entity_index = self.entity_index.get_mut();
        let mut location = entity_index[entity];
        if location == EntityLocation::uninitialized() {
            let empty_archetype = &mut self.archetypes[ArchetypeId::empty_archetype()];
            location.row = RowIndex(empty_archetype.entities.len());
            empty_archetype.entities.push(entity);
            entity_index[entity] = location;
        }
        location
    }

    pub fn entity_location(&mut self, entity: Entity) -> Option<EntityLocation> {
        let entity_index = self.entity_index.get_mut();
        entity_index.get(entity).copied()
    }

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

    /// Get metadata of a component
    pub(crate) fn component_info(&mut self, component: Entity) -> Option<ComponentInfo> {
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
                // edges: Default::default()
            };

            // Create columns
            for component in signature.iter() {
                // let info = self.component_info(component.as_entity().unwrap()).unwrap();
                // new_archetype.columns.push(RwLock::new(Column::new(info)));
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
            // self.connect_edges(signature, id)

            id
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
        }
        // add edge detection here {}
        else {
            self.create_archetype(current_archetype.signature.clone().with(info.id.into()))
        };

        // SAFETY: New chunk is immediately created for entity
        // Move entity
        // unsafe { self.move_entity(current_location, destination) };

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
}
