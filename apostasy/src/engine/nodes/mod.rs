use std::any::TypeId;

use crate::engine::nodes::component::Component;

pub mod camera;
pub mod component;
pub mod transform;
pub mod velocity;

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
    pub fn get_component_mut<T: Component + 'static>(&mut self) -> Option<&mut T> {
        self.components
            .iter_mut()
            .find(|component| component.as_any().type_id() == TypeId::of::<T>())
            .and_then(|component| component.as_any_mut().downcast_mut())
    }
    pub fn get_components_mut<'a, T: ComponentsMut<'a>>(&'a mut self) -> T {
        T::from_node(self)
    }
    pub fn add_component<T: Component + 'static>(&mut self, component: T) -> &mut Self {
        self.components.push(Box::new(component));
        self
    }
    pub fn add_child(&mut self, child: Node) -> &mut Self {
        self.children.push(child);
        self
    }
}
pub trait ComponentsMut<'a> {
    fn from_node(node: &'a mut Node) -> Self;
}

macro_rules! impl_components_mut {
    ($($T:ident),+) => {
        impl<'a, $($T: Component + 'static),+> ComponentsMut<'a> for ($(Option<&'a mut $T>),+) {
            fn from_node(node: &'a mut Node) -> Self {
                $(let mut $T: Option<*mut $T> = None;)+

                for component in node.components.iter_mut() {
                    let any = component.as_any_mut();
                    let type_id = any.type_id();
                    $(
                        if type_id == TypeId::of::<$T>() {
                            if let Some(v) = any.downcast_mut::<$T>() {
                                $T = Some(v as *mut $T);
                            }
                            continue;
                        }
                    )+
                }

                unsafe {
                    ($($T.map(|p| &mut *p)),+)
                }
            }
        }
    };
}

impl_components_mut!(A, B);
impl_components_mut!(A, B, C);
impl_components_mut!(A, B, C, D);
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
    pub fn add_node(&mut self, node: Node) -> &mut Self {
        self.root.add_child(node);
        self
    }

    pub fn get_all_nodes(&self) -> Vec<&Node> {
        let mut nodes = Vec::new();
        for node in self.root.children.iter() {
            nodes.push(node);
            nodes.extend(node.children.iter());
        }
        nodes
    }

    pub fn get_all_nodes_mut(&mut self) -> Vec<&mut Node> {
        let mut nodes = Vec::new();
        for node in self.root.children.iter_mut() {
            let node_ptr = node as *mut Node;
            unsafe {
                nodes.push(&mut *node_ptr);
                for child in (*node_ptr).children.iter_mut() {
                    nodes.push(child);
                }
            }
        }
        nodes
    }
}
