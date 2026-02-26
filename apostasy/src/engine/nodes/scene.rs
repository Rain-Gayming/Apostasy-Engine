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
        let mut root_node = Node::new();
        root_node.name = "root".to_string();
        Self {
            name: "Scene".to_string(),
            root_node: Box::new(root_node),
        }
    }
}
