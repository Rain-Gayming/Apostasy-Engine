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
pub mod resources;
pub mod systems;

#[derive(Default)]
pub struct ECSWorld {
    pub scheduler: Scheduler,
    pub entities: HashMap<u64, Entity>,
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
        self.entities.insert(entity_id, new_entity);
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
    pub fn add_component<T: Component>(
        &mut self,
        entity: &mut Entity,
        data: impl Any + Component,
    ) -> &mut Self {
        let type_id = TypeId::of::<T>();

        // loop thrrough archetypes,
        // find if an archetype contains all the components
        //   - [x] if it does
        //      - [x] add the current entity to it
        //   if not then
        //      create a new archetype with all the entities components
        //
        //   - [x] if the current entity id is found
        //      - [x] remove it from the archetype
        let mut has_found_new_archetype: bool = false;
        let mut column_builder: &mut ColumnsBuilder = &mut new_column_builder();
        for archetype in self.archetypes.iter_mut() {
            //   if the current entity id is found
            //      remove it from the archetype
            if archetype.entities.contains(entity) {
                // get the index of the entity
                let index = archetype
                    .entities
                    .iter()
                    .position(|&e| e.0 == entity.0)
                    .unwrap();

                // add all its components to the component builder
                for column in archetype.columns.iter_mut() {
                    let component_column: Box<dyn ComponentColumn> = column.new_empty_column();
                    column_builder = column_builder.add_column(component_column);
                }

                // remove the entity from the archetype
                archetype.entities.remove(index);
            }
        }
        for archetype in self.archetypes.iter_mut() {
            // does the current archetype contain the component we are adding
            // if it does
            //      add the current entity to it
            //
            for column in column_builder.0.iter() {
                if archetype.contains_component::<T>() {
                    archetype.entities.push(*entity);
                    has_found_new_archetype = true;
                    println!("adding to existing archetype");
                }
            }
        }

        if !has_found_new_archetype {
            let new_column_builder = column_builder.with_column_type::<T>();
            let mut new_archetype = new_archetype_from_builder(new_column_builder);
            new_archetype.entities.push(*entity);
            self.archetypes.push(new_archetype);
            println!("adding to new archetype");
        }

        // self.components.insert(type_id, Box::new(data));
        self
    }

    fn despawn(&mut self, entity: Entity) {
        if self.is_alive(entity) {
            self.dead_entities.insert(entity);
        }
    }

    fn is_alive(&self, entity: Entity) -> bool {
        if entity.0 >= self.next_entity_id {
            panic!("Attempted to use an entity in an EntityGenerator that it was not spawned with");
        }
        !self.dead_entities.contains(&entity)
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
