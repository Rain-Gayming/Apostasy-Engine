use anyhow::{Result, anyhow};
use slotmap::{DefaultKey, SlotMap};

use crate::{
    log_error,
    objects::{Object, component::Component, components::transform::Transform, tag::Tag},
    rendering::components::model_renderer::ModelRenderer,
};

pub type ObjectId = DefaultKey;
pub struct Scene {
    pub(crate) objects: SlotMap<ObjectId, Object>,
}

impl Default for Scene {
    fn default() -> Self {
        let mut scene = Scene {
            objects: SlotMap::new(),
        };

        scene.add_default_objects();

        scene
    }
}

impl Scene {
    pub(crate) fn add_default_objects(&mut self) {
        let test_model = Object::new()
            .add_component(Transform::default())
            .add_component(ModelRenderer::from_path("model.glb".to_string()))
            .set_name("Test Model".to_string())
            .clone();
        self.add_object(test_model);
    }
    /// Adds a new default Object and returns its ID
    pub fn add_new_object(&mut self) -> ObjectId {
        self.objects.insert(Object::default())
    }

    /// Adds an Object and returns its ID
    pub fn add_object(&mut self, object: Object) -> ObjectId {
        self.objects.insert(object)
    }

    /// Removes an Object by ID
    pub fn remove_object(&mut self, id: ObjectId) {
        if let Some(_) = self.objects.get(id) {
            self.objects.remove(id);
        } else {
            log_error!("Object does not exist!");
        }
    }

    pub fn debug_objects(&self) {
        for (id, object) in self.objects.iter() {
            println!("{}: {:?}", object.name, id);
        }
    }

    pub fn get_object(&self, id: ObjectId) -> Option<&Object> {
        if let Some(object) = self.objects.get(id) {
            return Some(object);
        }
        log_error!("Object does not exist!");
        None
    }

    pub fn get_object_mut(&mut self, id: ObjectId) -> Option<&mut Object> {
        self.objects.get_mut(id)
    }

    // ========== ========== Components ========== ==========
    pub fn get_objects_with_component<T: Component + 'static>(&self) -> Vec<&Object> {
        self.objects
            .values()
            .filter(|object| object.has_component::<T>())
            .collect()
    }

    pub fn get_objects_with_component_mut<T: Component + 'static>(&mut self) -> Vec<&mut Object> {
        self.objects
            .values_mut()
            .filter(|object| object.has_component::<T>())
            .collect()
    }

    pub fn get_objects_with_component_with_ids<T: Component + 'static>(
        &self,
    ) -> Vec<(ObjectId, &Object)> {
        self.objects
            .iter()
            .filter(|(_id, object)| object.has_component::<T>())
            .map(|(id, object)| (id, object))
            .collect()
    }

    // ========== ========== Tags ========== ==========
    pub fn get_object_with_tag<T: Tag + 'static>(&self) -> Result<&Object> {
        self.objects
            .values()
            .find(|object| object.has_tag::<T>())
            .ok_or(anyhow!("No objects with the tag"))
    }

    pub fn get_object_with_tag_mut<T: Tag + 'static>(&mut self) -> Result<&mut Object> {
        self.objects
            .values_mut()
            .find(|object| object.has_tag::<T>())
            .ok_or(anyhow!("No objects with the tag"))
    }

    pub fn get_objects_with_tag<T: Tag + 'static>(&self) -> Vec<&Object> {
        self.objects
            .values()
            .filter(|object| object.has_tag::<T>())
            .collect()
    }

    pub fn get_objects_with_tag_mut<T: Tag + 'static>(&mut self) -> Vec<&mut Object> {
        self.objects
            .values_mut()
            .filter(|object| object.has_tag::<T>())
            .collect()
    }
    pub fn get_objects_with_tag_with_ids<T: Tag + 'static>(&self) -> Vec<(ObjectId, &Object)> {
        self.objects
            .iter()
            .filter(|(_id, object)| object.has_tag::<T>())
            .collect()
    }
}
