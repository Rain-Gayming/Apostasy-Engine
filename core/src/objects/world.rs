use anyhow::Result;

use crate::objects::{
    Object,
    component::Component,
    resource::{Resource, ResourceMap},
    scene::Scene,
    systems::{FixedUpdateSystem, LateUpdateSystem, StartSystem, UpdateSystem},
    tag::Tag,
};

#[derive(Default)]
pub struct World {
    pub(crate) scene: Scene,
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
    pub fn add_new_object(&mut self) -> &mut Object {
        self.scene.add_new_object()
    }

    pub(crate) fn assign_object_ids(&mut self) {
        self.scene.assign_object_ids();
    }

    pub fn debug_objects(&self) {
        self.scene.debug_objects();
    }

    pub fn get_object(&self, id: u64) -> Option<&Object> {
        self.scene.get_object(id)
    }

    pub fn get_object_mut(&mut self, id: u64) -> Option<&mut Object> {
        self.scene.get_object_mut(id)
    }

    pub fn get_objects_with_component<T: Component + 'static>(&self) -> Vec<&Object> {
        self.scene.get_objects_with_component::<T>()
    }

    pub fn get_objects_with_component_mut<T: Component + 'static>(&mut self) -> Vec<&mut Object> {
        self.scene.get_objects_with_component_mut::<T>()
    }

    pub fn get_object_with_tag<T: Tag + 'static>(&self) -> Result<&Object> {
        self.scene.get_object_with_tag::<T>()
    }

    pub fn get_object_with_tag_mut<T: Tag + 'static>(&mut self) -> Result<&mut Object> {
        self.scene.get_object_with_tag_mut::<T>()
    }

    pub fn get_objects_with_tag<T: Tag + 'static>(&self) -> Vec<&Object> {
        self.scene.get_objects_with_tag::<T>()
    }

    pub fn get_objects_with_tag_mut<T: Tag + 'static>(&mut self) -> Vec<&mut Object> {
        self.scene.get_objects_with_tag_mut::<T>()
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
