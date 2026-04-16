use std::any::TypeId;

use anyhow::{Result, anyhow};
use gltf::camera;
use hashbrown::HashMap;

use crate::{
    log_error,
    objects::{Object, component::Component, components::transform::Transform, tag::Tag},
    physics::velocity::Velocity,
    rendering::components::{
        camera::{Camera, MainCamera},
        model_renderer::ModelRenderer,
    },
};

pub struct Scene {
    pub(crate) objects: HashMap<u64, Object>,
}

impl Default for Scene {
    fn default() -> Self {
        let mut scene = Scene {
            objects: HashMap::new(),
        };

        scene.add_default_objects();

        scene
    }
}

impl Scene {
    /// Adds a new Object to the world
    pub fn add_new_object(&mut self) -> &mut Object {
        let index = self.objects.len() as u64;
        self.objects.insert(index, Object::default());

        self.assign_object_ids();

        self.objects.get_mut(&index).unwrap()
    }

    pub fn add_object(&mut self, object: Object) {
        self.objects.insert(self.objects.len() as u64, object);
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

    pub fn get_object_with_tag_mut<T: Tag + 'static>(&mut self) -> Result<&mut Object> {
        println!("{}", T::type_name_static());
        self.objects
            .iter_mut()
            .find(|x| {
                x.1.tags
                    .iter()
                    .any(|tag| tag.type_name() == T::type_name_static())
            })
            .map(|x| x.1)
            .ok_or(anyhow!("No objects with the tag"))
    }

    pub fn add_default_objects(&mut self) {
        let camera_object = Object::new()
            .add_component(Transform::default())
            .add_component(Velocity::default())
            .add_component(Camera::default())
            .add_tag(MainCamera)
            .set_name("Camera".to_string())
            .clone();

        println!("{}", camera_object.tags.first().unwrap().type_name());

        self.add_object(camera_object);

        let test_model = Object::new()
            .add_component(Transform::default())
            .add_component(ModelRenderer::from_path("model.glb".to_string()))
            .set_name("Test Model".to_string())
            .clone();
        self.add_object(test_model);
    }
}
