use std::{
    any::{Any, TypeId},
    collections::{HashMap, HashSet},
};

use crate::app::engine::ecs::{
    archetype::{
        Archetype, ColumnsBuilder, ComponentColumn, new_archetype_from_builder, new_column_builder,
    },
    component::Component,
    entities::Entity,
    resources::Resource,
    systems::{IntoSystem, Scheduler, System},
};

pub mod archetype;
pub mod component;
pub mod components;
pub mod entities;
pub mod query;
pub mod resources;
pub mod systems;

#[derive(Default)]
pub struct ECSWorld {
    pub scheduler: Scheduler,
    pub entities: HashMap<Entity, (u64, u64)>,
    pub archetypes: Vec<Archetype>,
    pub next_entity_id: u64,
    pub dead_entities: HashSet<Entity>,
}

impl ECSWorld {
    /// Adds a resource to the resource pool
    /// ```
    /// fn add_resource(){
    ///     let mut world = ECSWorld::default();
    ///     let test_resource = TestResource(32.0);
    ///     world.add_resource(test_resource);
    /// }
    /// struct TestResource(f32);
    /// impl Resource for TestResource{}
    /// ```
    pub fn add_resource(&mut self, resource_data: impl Resource) {
        self.scheduler.add_resource(resource_data);
    }

    /// Removes a resource from the pool
    /// Query for a resource and get a mutable reference to it as an option
    /// allows for nothing to return but will cause a panic
    /// ```
    /// fn remove_resource(){
    ///     let mut world = ECSWorld::default();
    ///     let test_resource = TestResource(32.0);
    ///     world.add_resource(test_resource);
    ///
    ///     assert_eq(get_resource.0, 64.0);
    ///
    ///     world.remove_resource::<TestResource>();
    ///     assert(world.get_resource_ref::<TestResource>().is_none());
    /// }
    /// struct TestResource(f32);
    /// impl Resource for TestResource{}
    /// ```
    pub fn remove_resource<T: Resource>(&mut self) {
        let type_id = TypeId::of::<T>();
        self.scheduler.resources.remove(&type_id);
    }

    // /// Adds a blank entity to the entities pool,
    // /// to add components to it use
    // /// ```
    // /// fn foo(){
    // ///     let world = World::default();
    // ///     world.
    // ///     create_entity().with_component(xxx)
    // /// }
    // /// ```
    pub fn create_entity(&mut self) -> &mut Self {
        let entity_id: u64 = self.next_entity_id;

        if entity_id == u64::MAX {
            panic!("Attempted to spawn an entity after running out of IDs");
        }

        let new_entity = Entity(entity_id);

        // create an empty column builder and add the current component to it
        let empty_column_builder: &mut ColumnsBuilder = &mut new_column_builder();

        let mut has_found_new_archetype: bool = false;
        // loop through the archetypes
        for archetype in self.archetypes.iter_mut() {
            // does the current archetype contain the component we are adding
            // if it does
            //      add the current entity to it
            if archetype.contains_columns(&empty_column_builder.0) {
                archetype.entities.push(new_entity);
                has_found_new_archetype = true;
                // println!("adding to existing archetype");
            }
        }

        if !has_found_new_archetype {
            let mut new_archetype = new_archetype_from_builder(empty_column_builder);
            new_archetype.entities.push(new_entity);
            self.archetypes.push(new_archetype);
            // println!("adding to new archetype");
        }

        self.entities.insert(new_entity, (entity_id, 0));
        self.next_entity_id += 1;

        self
    }

    /// Adds a component to an entity
    ///
    /// ```
    /// fn create_entity() {
    ///     let mut world = ECSWorld::default();
    ///
    ///     let new_entity = world
    ///         .create_entity()
    ///         .add_component::<NewComponent>(NewComponent(59.0))
    ///         .add_component::<NewComponentB>(NewComponentB(590.0));
    /// }
    /// ```
    pub fn add_component<T: Component + PartialEq + Clone>(
        &mut self,
        entity: &mut Entity,
        data: impl Any + Component,
    ) -> &mut Self {
        // create an empty column builder and add the current component to it
        let empty_column_builder: &mut ColumnsBuilder = &mut new_column_builder();
        let mut column_builder = empty_column_builder.with_column_type::<T>();
        let mut components: Vec<Box<dyn Component>> = vec![Box::new(data)];

        // loop through all archetypes
        for archetype in self.archetypes.iter_mut() {
            if let Some(index) = archetype.entities.iter().position(|&e| e.0 == entity.0) {
                // add all its components to the component builder
                for column in archetype.columns.iter_mut() {
                    let component_column: Box<dyn ComponentColumn> = column.new_empty_column();
                    column_builder = column_builder.add_column(component_column);
                }

                // for each component type
                for component_type in archetype.component_types.iter() {
                    // get all components with this type
                    if let Some(component_vec) = archetype.components.get(component_type) {
                        // get the component relating to the entity
                        if let Some(component) = component_vec.get(index) {
                            components.push(component.clone());
                        }
                    }
                }

                // remove the entity from the archetype
                archetype.entities.remove(index);
                break; // entity should only be in one archetype
            }
        }

        // loop through the archetypes again
        // TODO: convert this to be better
        for pos in 0..self.archetypes.len() {
            // does the current archetype contain the component we are adding
            // if it does
            //      add the current entity to it
            let archetype = self.archetypes.get_mut(pos).unwrap();
            if archetype.contains_columns(&column_builder.0) {
                archetype.entities.push(*entity);

                // add the components to the archetype
                for component in components {
                    let component_vec = archetype
                        .components
                        .get_mut(&(*component).type_id())
                        .unwrap();
                    component_vec.push(component);
                }

                // update the archetype id on the entity to the current archetype
                let entity = self.entities.get_mut(entity).unwrap();
                entity.0 = pos as u64;
                return self;
            }
        }

        // if an archetype fitting the entities components wasnt found
        // create a new one
        for component in components {
            column_builder.1.push(component);
        }
        let mut new_archetype = new_archetype_from_builder(column_builder);
        new_archetype.entities.push(*entity);

        self.archetypes.push(new_archetype);

        // update the entities archetype id to the newest added one
        let entity = self.entities.get_mut(entity).unwrap();
        entity.0 = self.archetypes.len() as u64 - 1;
        println!("adding to new archetype");

        self
    }
    /// Adds a component to an entity
    /// **ONLY USE WHILE ADDING TO A NEWLY CREATED ENTITY**
    /// ```
    /// fn create_entity() {
    ///     let mut world = ECSWorld::default();
    ///
    ///     let new_entity = world
    ///         .create_entity()
    ///         .with_component::<NewComponent>(NewComponent(59.0))
    ///         .with_component::<NewComponentB>(NewComponentB(590.0));
    /// }
    /// ```
    pub fn with_component<T: Component + PartialEq + Clone>(
        &mut self,
        data: impl Any + Component,
    ) -> &mut Self {
        // get the last entity in the array (should be the most recently added)
        let last_entity_id = self.entities.len() - 1;
        let mut entity = Entity(last_entity_id as u64);

        // add the component
        self.add_component::<T>(&mut entity, data);

        self
    }

    /// Takes in a comopnent and an entity,
    /// returns a reference to the component on that entity if it has it
    pub fn get_component<T: Component>(&self, entity: Entity) -> Option<&T> {
        let (archetype_idx, entity_idx) = self.entities.get(&entity)?;
        let archetype = &self.archetypes[*archetype_idx as usize];

        let component_vec: &_ = archetype.components.get(&TypeId::of::<T>())?;
        component_vec
            .get(*entity_idx as usize)
            .unwrap()
            .downcast_ref()
    }

    /// Takes in a comopnent and an entity,
    /// returns a mutable reference component on that entity if it has it
    pub fn get_component_mut<T: Component>(&mut self, entity: Entity) -> Option<&mut T> {
        let (archetype_idx, entity_idx) = self.entities.get(&entity)?;
        let archetype = &mut self.archetypes[*archetype_idx as usize];

        let component_vec = archetype.components.get_mut(&TypeId::of::<T>())?;

        component_vec
            .get_mut(*entity_idx as usize)
            .unwrap()
            .downcast_mut()
    }

    /// Adds a system to the world
    /// ```
    /// fn add_system_test() {
    ///     let mut world = ECSWorld::default();
    ///     world.add_system(test_system());
    /// }
    /// fn test_system() {
    ///     println!("test");
    /// }
    /// ```
    pub fn add_system<I, S: System + 'static>(
        &mut self,
        system: impl IntoSystem<I, System = S>,
    ) -> &mut Self {
        self.scheduler.add_system(system);
        self
    }

    /// Runs all per-frame systems set in the world
    ///```
    /// fn add_system_test() {
    ///     let mut world = ECSWorld::default();
    ///     world.add_system(test_system());
    ///     world.run();
    /// }
    /// fn test_system() {
    ///     println!("test");
    /// }
    /// ```
    pub fn run(&mut self) {
        for system in self.scheduler.systems.iter_mut() {
            system.run(&mut self.scheduler.resources);
        }
    }
}
