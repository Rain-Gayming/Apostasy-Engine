use std::{collections::HashMap, mem::MaybeUninit};

use parking_lot::{
    MappedRwLockReadGuard, MappedRwLockWriteGuard, Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard,
};

use crate::{
    engine::ecs::{
        archetype::{
            Archetype, ArchetypeEdge, ArchetypeId, Column, ColumnIndex, RowIndex, Signature,
        },
        component::{COMPONENT_ENTRIES, Component, ComponentId, ComponentInfo, ComponentLocations},
        entity::{Entity, EntityLocation},
    },
    utils::slotmap::SlotMap,
};

/// The core of the ecs, it contains the archetypes and entities
pub struct Core {
    pub archetypes: SlotMap<ArchetypeId, Archetype>,
    pub entity_index: Mutex<SlotMap<Entity, EntityLocation>>,
    pub component_index: HashMap<ComponentId, ComponentLocations>,
    pub signature_index: HashMap<Signature, ArchetypeId>,
}

#[allow(clippy::new_without_default)]
impl Core {
    /// Creates a new core, not manually called
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

    /// Returns an entity's location
    pub fn get_entity_location_locking(&self, entity: Entity) -> Option<EntityLocation> {
        let entity_index = self.entity_index.lock();
        entity_index.get(entity).copied()
    }

    /// Creates a new entity
    pub fn create_unspawned_entity(&self) -> Entity {
        let mut entity_index = self.entity_index.lock();
        entity_index.insert(EntityLocation::uninitialized())
    }

    /// Spawns an entity in the empty archetype if it is uninitialized
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

    #[allow(clippy::clone_on_copy, unused_assignments)]
    pub fn despawn_entity(&mut self, entity: Entity) {
        let entity_index = self.entity_index.get_mut();
        let location = entity_index[entity];

        let archetype = &mut self.archetypes[location.archetype];
        let mut current_row = archetype.entities.get(location.row.0).unwrap();
        let mut final_row = archetype.entities.last().unwrap();

        // Only swap if the current row isnt the last row
        if current_row != final_row {
            let stored_row = final_row.clone();
            final_row = current_row;
            current_row = &stored_row;
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

    /// Creates a new Archetype with the given Signature
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

    /// Connects the edges of an Archetype
    fn connect_edges(&mut self, signature: Signature, id: ArchetypeId) {
        for component in signature.iter() {
            let without_component = signature.clone().without(*component);
            let Some(other) = self.signature_index.get(&without_component).copied() else {
                continue;
            };

            // Connect this to other
            self.archetypes[id]
                .edges
                .entry(*component)
                .or_default()
                .remove = Some(other);

            // Connect other to this
            self.archetypes[other]
                .edges
                .entry(*component)
                .or_default()
                .add = Some(id);
        }
    }

    /// Inserts a Component and it's data into an Entity
    #[allow(clippy::missing_safety_doc)]
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

    /// Moves an Entity between two Archetypes
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

    /// Removes a component from an Entity
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

    /// Get a component from an entity as type erased bytes
    pub fn get_bytes<'a>(
        &'a self,
        component: ComponentId,
        entity_location: EntityLocation,
    ) -> Option<MappedRwLockReadGuard<'a, [MaybeUninit<u8>]>> {
        self.component_index
            .get(&component)
            .and_then(|component_locations| {
                let column = self
                    .archetypes
                    .get(entity_location.archetype)?
                    .columns
                    .get(**component_locations.get(&entity_location.archetype)?)?
                    .read();
                Some(RwLockReadGuard::map(column, |column| {
                    column.get_chunk(entity_location.row)
                }))
            })
    }

    /// Get a component from an entity as type erased bytes
    pub fn get_bytes_mut<'a>(
        &'a self,
        component: ComponentId,
        entity_location: EntityLocation,
    ) -> Option<MappedRwLockWriteGuard<'a, [MaybeUninit<u8>]>> {
        self.component_index
            .get(&component)
            .and_then(|component_locations| {
                let column = self
                    .archetypes
                    .get(entity_location.archetype)?
                    .columns
                    .get(**component_locations.get(&entity_location.archetype)?)?
                    .write();
                Some(RwLockWriteGuard::map(column, |column| {
                    column.get_chunk_mut(entity_location.row)
                }))
            })
    }
}
