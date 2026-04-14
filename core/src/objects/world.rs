use anyhow::Result;
use hashbrown::HashMap;

use crate::{
    log_error,
    objects::{
        Object,
        component::Component,
        resource::{Resource, ResourceMap},
        systems::{FixedUpdateSystem, LateUpdateSystem, StartSystem, UpdateSystem},
    },
};

#[derive(Default)]
pub struct World {
    pub(crate) objects: HashMap<u64, Object>,
    pub(crate) resources: ResourceMap,
}

#[allow(unused)]
impl World {
    // ========== ========== Systems ========== ==========

    /// Runs all start systems
    pub(crate) fn start(&mut self) {
        let mut systems = inventory::iter::<StartSystem>().collect::<Vec<_>>();
        systems.sort_by(|a, b| a.priority.cmp(&b.priority));
        systems.reverse();
        for system in systems.iter_mut() {
            (system.func)(self);
        }
    }

    /// Runs all update systems
    pub(crate) fn update(&mut self) {
        let mut systems = inventory::iter::<UpdateSystem>().collect::<Vec<_>>();
        systems.sort_by(|a, b| a.priority.cmp(&b.priority));
        systems.reverse();
        for system in systems.iter_mut() {
            (system.func)(self);
        }
    }

    /// Runs all fixed update systems
    pub(crate) fn fixed_update(&mut self, delta: f32) {
        let mut systems = inventory::iter::<FixedUpdateSystem>().collect::<Vec<_>>();
        systems.sort_by(|a, b| a.priority.cmp(&b.priority));
        systems.reverse();
        for system in systems.iter_mut() {
            (system.func)(self, delta);
        }
    }

    /// Runs all late update systems
    pub(crate) fn late_update(&mut self) {
        let mut systems = inventory::iter::<LateUpdateSystem>().collect::<Vec<_>>();
        systems.sort_by(|a, b| a.priority.cmp(&b.priority));
        systems.reverse();
        for system in systems.iter_mut() {
            (system.func)(self);
        }
    }

    // ========== ========== Objects ========== ==========
    /// Adds a new Object to the world
    pub fn add_new_object(&mut self) -> u64 {
        let index = self.objects.len();
        self.objects.insert(index as u64, Object::default());

        self.assign_object_ids();

        index as u64
    }

    pub(crate) fn assign_object_ids(&mut self) {
        let mut index = 0;

        for object in self.objects.iter_mut() {
            object.1.id = index;
            index += 1;
        }
    }

    pub fn debug_objects(&self) {
        for object in self.objects.iter() {
            println!("{}: {}", object.1.name, object.1.id);
        }
    }

    pub fn get_object(&self, id: u64) -> Option<&Object> {
        if let Some(object) = self.objects.get(&id) {
            return Some(object);
        }

        log_error!("Object: {} does not exist!", id.to_string());
        return None;
    }

    pub fn get_object_mut(&mut self, id: u64) -> Option<&mut Object> {
        if let Some(object) = self.objects.get_mut(&id) {
            return Some(object);
        }

        log_error!("Object: {} does not exist!", id.to_string());
        return None;
    }

    pub fn get_objects_with_component<T: Component + 'static>(&self) -> Vec<&Object> {
        let mut objects: Vec<&Object> = Vec::new();

        self.objects.iter().for_each(|(_id, object)| {
            if object.has_component::<T>() {
                objects.push(&object);
            }
        });

        objects
    }

    pub fn get_objects_with_component_mut<T: Component + 'static>(&mut self) -> Vec<&mut Object> {
        let mut objects: Vec<&mut Object> = Vec::new();

        self.objects.iter_mut().for_each(|(_id, object)| {
            if object.has_component::<T>() {
                objects.push(object);
            }
        });

        objects
    }

    // ========== ========== Resources ========== ==========

    /// Insert a new resource into the map
    pub fn insert_resource<T: Resource + 'static>(&mut self, resource: T) -> &mut Self {
        self.resources.insert(resource);
        self
    }

    /// Get a resource from the map
    pub fn get_resource<T: Resource + 'static>(&self) -> Result<&T> {
        self.resources.get::<T>()
    }

    /// Get a resource mutably from the map
    pub fn get_resource_mut<T: Resource + 'static>(&mut self) -> Result<&mut T> {
        self.resources.get_mut::<T>()
    }

    /// Remove a resource from the map
    pub fn remove_resource<T: Resource + 'static>(&mut self) -> &mut Self {
        self.resources.remove::<T>();
        self
    }
}
