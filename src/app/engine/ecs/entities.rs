use std::any::Any;

use crate::app::engine::ecs::component::Component;

pub struct Entity {
    pub components: Vec<Box<dyn Component>>,
    pub id: u32,
}

impl Entity {
    pub fn with_component(&mut self, data: impl Any + Component) -> &mut Self {
        self.components.push(Box::new(data));
        self
    }
}
