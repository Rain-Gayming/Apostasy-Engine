use std::any::TypeId;

use crate::engine::nodes::component::Component;

pub mod component;

#[derive(Clone)]
pub struct Node {
    pub name: String,
    pub children: Vec<Node>,
    pub parent: Option<Box<Node>>,
    pub components: Vec<Box<dyn Component>>,
}

impl Default for Node {
    fn default() -> Self {
        Self::new()
    }
}

impl Node {
    pub fn new() -> Self {
        Self {
            name: "Node".to_string(),
            children: Vec::new(),
            parent: None,
            components: Vec::new(),
        }
    }

    pub fn get_component<T: Component + 'static>(&self) -> Option<&T> {
        self.components
            .iter()
            .find(|component| component.as_any().type_id() == TypeId::of::<T>())
            .and_then(|component| component.as_any().downcast_ref())
    }
}

pub struct World {
    pub root: Box<Node>,
    pub global_nodes: Vec<Box<Node>>,
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

impl World {
    pub fn new() -> Self {
        Self {
            root: Box::new(Node::new()),
            global_nodes: Vec::new(),
        }
    }

    pub fn add_global_node(&mut self, node: Box<Node>) {
        self.global_nodes.push(node);
    }
}
