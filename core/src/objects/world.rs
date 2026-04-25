use anyhow::Result;

use crate::objects::{
    Object,
    component::Component,
    resource::{Resource, ResourceMap},
    scene::{ObjectId, Scene},
    systems::{FixedUpdateSystem, HasPriority, LateUpdateSystem, StartSystem, UpdateSystem},
    tag::Tag,
};

#[derive(Default)]
pub struct World {
    pub(crate) scene: Scene,
    pub(crate) resources: ResourceMap,

    update_systems: Vec<&'static UpdateSystem>,
    fixed_update_systems: Vec<&'static FixedUpdateSystem>,
    late_update_systems: Vec<&'static LateUpdateSystem>,
}

#[allow(unused)]
impl World {
    // ========== ========== Systems ========== ==========

    /// Collects and caches all systems
    pub fn build_systems(&mut self) {
        self.update_systems = Self::collect_sorted(inventory::iter::<UpdateSystem>());
        self.fixed_update_systems = Self::collect_sorted(inventory::iter::<FixedUpdateSystem>());
        self.late_update_systems = Self::collect_sorted(inventory::iter::<LateUpdateSystem>());
    }

    /// Collects and sorts the Iterator
    fn collect_sorted<T: HasPriority>(iter: impl Iterator<Item = &'static T>) -> Vec<&'static T> {
        let mut systems: Vec<_> = iter.collect();
        systems.sort_by(|a, b| b.priority().cmp(&a.priority()));

        systems
    }
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
        let systems = std::mem::take(&mut self.update_systems);
        for system in &systems {
            (system.func)(self);
        }
        self.update_systems = systems;
    }

    /// Runs all fixed update systems
    pub(crate) fn fixed_update(&mut self, delta: f32) {
        let systems = std::mem::take(&mut self.fixed_update_systems);
        for system in &systems {
            (system.func)(self, delta);
        }
        self.fixed_update_systems = systems;
    }

    /// Runs all late update systems
    pub(crate) fn late_update(&mut self) {
        let systems = std::mem::take(&mut self.late_update_systems);
        for system in &systems {
            (system.func)(self);
        }
        self.late_update_systems = systems;
    } // ========== ========== Objects ========== ==========

    /// Adds a new Object to the world
    pub fn add_new_object(&mut self) -> ObjectId {
        self.scene.add_new_object()
    }

    /// Adds an Object to the world
    pub fn add_object(&mut self, object: Object) {
        self.scene.add_object(object);
    }

    /// Removes an Object from the world
    pub fn remove_object(&mut self, id: ObjectId) {
        self.scene.remove_object(id);
    }

    pub fn debug_objects(&self) {
        self.scene.debug_objects();
    }

    pub fn get_object(&self, id: ObjectId) -> Option<&Object> {
        self.scene.get_object(id)
    }

    pub fn get_object_mut(&mut self, id: ObjectId) -> Option<&mut Object> {
        self.scene.get_object_mut(id)
    }

    pub fn get_objects_with_component_with_ids<T: Component + 'static>(
        &self,
    ) -> Vec<(ObjectId, &Object)> {
        self.scene.get_objects_with_component_with_ids::<T>()
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
    pub fn get_objects_with_tag_with_ids<T: Tag + 'static>(&self) -> Vec<(ObjectId, &Object)> {
        self.scene.get_objects_with_tag_with_ids::<T>()
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
