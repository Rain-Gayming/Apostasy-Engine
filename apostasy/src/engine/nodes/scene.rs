use crate::engine::nodes::Node;

pub struct Scene {
    pub name: String,
    pub root_node: Box<Node>,
}

impl Default for Scene {
    fn default() -> Self {
        Self::new()
    }
}

impl Scene {
    pub fn new() -> Self {
        Self {
            name: "Scene".to_string(),
            root_node: Box::new(Node::new()),
        }
    }
}
