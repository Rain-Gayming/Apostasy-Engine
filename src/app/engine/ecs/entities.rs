use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

use crate::app::engine::ecs::component::Component;

pub struct Entity {
    pub components: HashMap<TypeId, Box<dyn Component>>,
    pub id: u32,
}

impl Entity {
    pub fn with_component<T: Component>(&mut self, data: impl Any + Component) -> &mut Self {
        let type_id = TypeId::of::<T>();

        self.components.insert(type_id, Box::new(data));
        self
    }

    pub fn get_component_ref<T: Component>(&self) -> Option<&T> {
        let type_id = TypeId::of::<T>();
        if let Some(data) = self.components.get(&type_id) {
            data.downcast_ref()
        } else {
            None
        }
    }
}
