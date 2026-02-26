use crate as apostasy;
use std::any::TypeId;

use anyhow::Result;
use apostasy_macros::start;
use cgmath::{Rotation, Vector3};

use crate::engine::{
    nodes::{
        component::Component,
        components::transform::{ParentGlobal, Transform},
        scene::Scene,
        scene_serialization::{SerializedScene, deserialize_node, serialize_node},
        system::{FixedUpdateSystem, InputSystem, LateUpdateSystem, StartSystem, UpdateSystem},
    },
    windowing::input_manager::InputManager,
};

pub mod component;
pub mod components;
pub mod scene;
pub mod scene_serialization;
pub mod system;

#[derive(Clone)]
pub struct Node {
    pub name: String,
    pub editing_name: String,
    pub children: Vec<Node>,
    pub parent: Option<String>,
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
            editing_name: "Node".to_string(),
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
    pub fn add_child(&mut self, mut child: Node) -> &mut Self {
        child.parent = Some(self.name.clone());
        self.children.push(child);
        self
    }

    pub fn propagate_transform(&mut self, parent: Option<&ParentGlobal>) {
        let binding = ParentGlobal::default();
        let parent = parent.unwrap_or(&binding);

        if let Some(t) = self.get_component_mut::<Transform>() {
            let global_position = parent.position
                + parent.rotation.rotate_vector(Vector3::new(
                    t.position.x * parent.scale.x,
                    t.position.y * parent.scale.y,
                    t.position.z * parent.scale.z,
                ));
            let global_rotation = parent.rotation * t.rotation;
            let global_scale = Vector3::new(
                parent.scale.x * t.scale.x,
                parent.scale.y * t.scale.y,
                parent.scale.z * t.scale.z,
            );

            t.global_position = global_position;
            t.global_rotation = global_rotation;
            t.global_scale = global_scale;
        }

        // Collect the new globals to pass to children
        let my_global = self
            .get_component::<Transform>()
            .map(|t| ParentGlobal {
                position: t.global_position,
                rotation: t.global_rotation,
                scale: t.global_scale,
            })
            .unwrap_or_else(|| parent.clone());

        for child in self.children.iter_mut() {
            child.propagate_transform(Some(&my_global));
        }
    }
}
pub trait ComponentsMut<'a> {
    fn from_node(node: &'a mut Node) -> Self;
}
macro_rules! impl_components_mut {
    ($($T:ident),+) => {

        #[allow(nonstandard_style)]
        impl<'a, $($T: Component + 'static),+> ComponentsMut<'a> for ($(&'a mut $T),+) {
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
                    ($(
                        $T.map(|p| &mut *p)
                            .unwrap_or_else(|| panic!("Error: Component ({}) not found on node", std::any::type_name::<$T>()))
                    ),+)
                }
            }
        }
    };
}
impl_components_mut!(A, B);
impl_components_mut!(A, B, C);
impl_components_mut!(A, B, C, D);

pub struct World {
    pub scene: Scene,
    pub global_nodes: Vec<Node>,
    pub input_manager: InputManager,
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

impl World {
    pub fn new() -> Self {
        Self {
            scene: Scene::new(),
            global_nodes: Vec::new(),
            input_manager: InputManager::default(),
        }
    }

    pub fn add_global_node(&mut self, node: Node) {
        self.global_nodes.push(node);
    }
    pub fn add_node(&mut self, node: Node) -> &mut Self {
        self.scene.root_node.add_child(node);
        self
    }

    pub fn add_new_node(&mut self) -> &mut Self {
        self.add_node(Node::new());

        self
    }

    pub fn get_all_nodes(&self) -> Vec<&Node> {
        fn collect<'a>(node: &'a Node, out: &mut Vec<&'a Node>) {
            out.push(node);
            for child in &node.children {
                collect(child, out);
            }
        }

        let mut nodes = Vec::new();
        for node in self.scene.root_node.children.iter() {
            collect(node, &mut nodes);
        }
        nodes
    }

    pub fn get_all_nodes_mut(&mut self) -> Vec<&mut Node> {
        fn collect<'a>(node: &'a mut Node, out: &mut Vec<*mut Node>) {
            out.push(node as *mut Node);
            for child in node.children.iter_mut() {
                collect(child, out);
            }
        }

        let mut ptrs: Vec<*mut Node> = Vec::new();
        for node in self.scene.root_node.children.iter_mut() {
            collect(node, &mut ptrs);
        }

        // SAFETY: each pointer is a unique node in the tree; we never alias.
        unsafe { ptrs.into_iter().map(|p| &mut *p).collect() }
    }

    pub fn get_node_with_name(&self, name: &str) -> &Node {
        self.get_all_nodes()
            .into_iter()
            .find(|node| node.name == name)
            .unwrap()
    }

    pub fn get_node_with_name_mut(&mut self, name: &str) -> &mut Node {
        self.get_all_nodes_mut()
            .into_iter()
            .find(|node| node.name == name)
            .unwrap()
    }
    pub fn start(&mut self) {
        let mut systems = inventory::iter::<StartSystem>().collect::<Vec<_>>();
        systems.sort_by(|a, b| a.priority.cmp(&b.priority));
        systems.reverse();
        for system in systems.iter_mut() {
            (system.func)(self);
        }
    }
    pub fn update(&mut self) {
        let mut systems = inventory::iter::<UpdateSystem>().collect::<Vec<_>>();
        systems.sort_by(|a, b| a.priority.cmp(&b.priority));
        systems.reverse();
        for system in systems.iter_mut() {
            (system.func)(self);
        }
    }

    pub fn fixed_update(&mut self, delta: f32) {
        let mut systems = inventory::iter::<FixedUpdateSystem>().collect::<Vec<_>>();
        systems.sort_by(|a, b| a.priority.cmp(&b.priority));
        systems.reverse();
        for system in systems.iter_mut() {
            (system.func)(self, delta);
        }
    }
    pub fn late_update(&mut self) {
        let mut systems = inventory::iter::<LateUpdateSystem>().collect::<Vec<_>>();
        systems.sort_by(|a, b| a.priority.cmp(&b.priority));
        systems.reverse();
        for system in systems.iter_mut() {
            (system.func)(self);
        }
    }
    pub fn get_node_with_component<T: Component + 'static>(&self) -> &Node {
        let node = self
            .get_all_nodes()
            .into_iter()
            .find(|node| node.get_component::<T>().is_some());

        if node.is_none() {
            panic!(
                "No node with component ({}) found",
                std::any::type_name::<T>()
            );
        }
        node.unwrap()
    }

    pub fn get_node_with_component_mut<T: Component + 'static>(&mut self) -> &mut Node {
        let node = self
            .get_all_nodes_mut()
            .into_iter()
            .find(|node| node.get_component::<T>().is_some());

        if node.is_none() {
            panic!(
                "No node with component ({}) found",
                std::any::type_name::<T>()
            );
        }
        node.unwrap()
    }

    pub fn serialize_scene(&self) -> Result<(), std::io::Error> {
        let serialized = SerializedScene {
            root_children: self
                .scene
                .root_node
                .children
                .iter()
                .map(serialize_node)
                .collect(),
        };
        let path = format!("{}/{}.yaml", ENGINE_SCENE_SAVE_PATH, self.scene.name);
        std::fs::write(path, serde_yaml::to_string(&serialized).unwrap())
    }

    pub fn deserialize_scene(&mut self, scene: String) -> Result<(), serde_yaml::Error> {
        let path = format!("{}/{}.yaml", ENGINE_SCENE_SAVE_PATH, scene);
        let contents = std::fs::read_to_string(&path).expect("Failed to read scene file");
        let serialized: SerializedScene = serde_yaml::from_str(&contents)?;
        self.scene.root_node.children = serialized
            .root_children
            .into_iter()
            .map(deserialize_node)
            .collect();
        Ok(())
    }
}
const ENGINE_SCENE_SAVE_PATH: &str = "res/scenes";

#[start]
pub fn start_system(world: &mut World) {
    world.input_manager.deserialize_input_manager().unwrap();
}
