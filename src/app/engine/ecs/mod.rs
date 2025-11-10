use std::{any::TypeId, collections::HashMap};

use crate::app::engine::ecs::{entities::Entity, resources::Resource};

pub mod component;
pub mod components;
pub mod entities;
pub mod resources;

#[derive(Default)]
pub struct ECSWorld {
    pub resources: HashMap<TypeId, Box<dyn Resource>>,
    pub entities: HashMap<u32, Entity>,
    pub systems: Vec<()>,
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
        let type_id = resource_data.type_id();
        self.resources.insert(type_id, Box::new(resource_data));
    }

    /// Query for a resource and get a non-mutable reference to it as an option
    /// allows for nothing to return but will cause a panic
    /// ```
    /// fn get_resource_ref(){
    ///     let mut world = ECSWorld::default();
    ///     let test_resource = TestResource(32.0);
    ///     world.add_resource(test_resource);
    ///
    ///     let get_resource = world.get_resource_ref::<TestResource>().unwrap();
    ///     assert_eq(get_resource.0, 32.0);
    /// }
    /// struct TestResource(f32);
    /// impl Resource for TestResource{}
    /// ```
    pub fn get_resource_ref<T: Resource>(&self) -> Option<&T> {
        let type_id = TypeId::of::<T>();
        if let Some(data) = self.resources.get(&type_id) {
            data.downcast_ref()
        } else {
            None
        }
    }

    /// Query for a resource and get a mutable reference to it as an option
    /// allows for nothing to return but will cause a panic
    /// ```
    /// fn get_resource_mut(){
    ///     let mut world = ECSWorld::default();
    ///     let test_resource = TestResource(32.0);
    ///     world.add_resource(test_resource);
    ///
    ///     let get_resource = world.get_resource_mut::<TestResource>().unwrap();
    ///     get_resource.0 += 32.0;
    ///     
    ///     assert_eq(get_resource.0, 64.0);
    /// }
    /// struct TestResource(f32);
    /// impl Resource for TestResource{}
    /// ```
    pub fn get_resource_mut<T: Resource>(&mut self) -> Option<&mut T> {
        let type_id = TypeId::of::<T>();
        if let Some(data) = self.resources.get_mut(&type_id) {
            data.downcast_mut()
        } else {
            None
        }
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
        self.resources.remove(&type_id);
    }

    /// Adds a blank entity to the entities pool,
    /// to add components to it use
    /// ```
    /// fn foo(){
    ///     let world = World::default();
    ///     world.
    ///     create_entity().with_component(xxx)
    /// }
    /// ```
    pub fn create_entity(&mut self) -> &mut Entity {
        let entity_id: u32 = self.entities.len() as u32;

        let new_entity = Entity {
            components: HashMap::new(),
            id: entity_id,
        };
        self.entities.insert(entity_id, new_entity);

        self.entities.get_mut(&entity_id).unwrap()
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
    pub fn add_system(&mut self, system: ()) -> &mut Self {
        self.systems.push(system);
        self
    }

    /// Runs all per-frame systems set in the world
    ///```
    /// fn add_system_test() {
    ///     let mut world = ECSWorld::default();
    ///     world.add_system(test_system());
    ///     world.run_systems();
    /// }
    /// fn test_system() {
    ///     println!("test");
    /// }
    /// ```
    pub fn run_systems(&self) {
        for system in self.systems.iter() {
            system;
        }
    }
}
