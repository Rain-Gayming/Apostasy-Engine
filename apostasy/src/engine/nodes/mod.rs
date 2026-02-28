use crate::{
    self as apostasy,
    engine::nodes::{
        scene::SceneManager, scene_serialization::find_registration,
        system::EditorFixedUpdateSystem,
    },
    log, log_warn,
};
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
        system::{FixedUpdateSystem, LateUpdateSystem, StartSystem, UpdateSystem},
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
    pub id: u64,
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
            id: 0,
            editing_name: "Node".to_string(),
            children: Vec::new(),
            parent: None,
            components: Vec::new(),
        }
    }

    /// Checks if the node has a component of type T
    /// ```rust
    ///     world.add_node(Node::new());
    ///     world.get_node_with_component::<Transform>().has_component::<Transform>();
    /// ```
    pub fn has_component<T: Component + 'static>(&self) -> bool {
        self.components
            .iter()
            .find(|component| component.as_any().type_id() == TypeId::of::<T>())
            .is_some()
    }

    /// Gets a component of type T from the node
    pub fn get_component<T: Component + 'static>(&self) -> Option<&T> {
        self.components
            .iter()
            .find(|component| component.as_any().type_id() == TypeId::of::<T>())
            .and_then(|component| component.as_any().downcast_ref())
    }

    /// Gets a mutable component of type T from the node
    pub fn get_component_mut<T: Component + 'static>(&mut self) -> Option<&mut T> {
        self.components
            .iter_mut()
            .find(|component| component.as_any().type_id() == TypeId::of::<T>())
            .and_then(|component| component.as_any_mut().downcast_mut())
    }

    /// Gets mutable components of type (T, T, ...) from the node
    pub fn get_components_mut<'a, T: ComponentsMut<'a>>(&'a mut self) -> T {
        T::from_node(self)
    }

    /// Adds a component of type T to the node
    pub fn add_component<T: Component + 'static>(&mut self, component: T) -> &mut Self {
        self.components.push(Box::new(component));
        self
    }

    /// Adds a child to the node
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

    // Remove a node by name from anywhere in the tree, returning it
    pub fn remove_node(&mut self, id: u64) -> Option<Node> {
        if let Some(pos) = self.children.iter().position(|c| c.id == id) {
            return Some(self.children.remove(pos));
        }
        for child in self.children.iter_mut() {
            if let Some(found) = child.remove_node(id) {
                return Some(found);
            }
        }
        None
    }

    // Insert a node as a child of the node with the given name
    pub fn insert_under(&mut self, parent_id: u64, mut node: Node) -> bool {
        if self.id == parent_id {
            node.parent = Some(self.name.clone());
            self.children.push(node);
            return true;
        }
        for child in self.children.iter_mut() {
            if child.insert_under(parent_id, node.clone()) {
                return true; // a bit wasteful due to clone, see note below
            }
            // note: ideally use Option passing to avoid clone, this is simplified
        }
        false
    }

    pub fn add_component_by_name(&mut self, component_name: &str) -> Result<()> {
        let registration =
            find_registration(component_name.to_lowercase().as_str()).ok_or_else(|| {
                log_warn!("Component '{}' is not registered", component_name);
                anyhow::anyhow!(
                    "Component '{}' is not registered",
                    component_name.to_lowercase()
                )
            })?;

        let component = (registration.create)();

        self.components.push(component);
        Ok(())
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
    pub scene_manager: SceneManager,
    pub nodes: u64,
    pub global_nodes: Vec<Node>,
    pub input_manager: InputManager,
    pub is_world_hovered: bool,
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
            scene_manager: SceneManager::new(),
            nodes: 0,
            global_nodes: Vec::new(),
            input_manager: InputManager::default(),
            is_world_hovered: false,
        }
    }

    pub fn add_global_node(&mut self, node: Node) {
        self.global_nodes.push(node);
        self.check_node_ids();
    }
    pub fn add_node(&mut self, mut node: Node) -> &mut Self {
        self.assign_ids_recursive(&mut node);
        self.scene.root_node.add_child(node);
        self.check_node_names();
        self
    }

    pub fn add_new_node(&mut self) {
        self.add_node(Node::new());
        self.check_node_names();
        self.check_node_ids();
    }

    pub fn get_all_world_nodes(&self) -> Vec<&Node> {
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

    pub fn get_all_world_nodes_mut(&mut self) -> Vec<&mut Node> {
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

        for node in self.global_nodes.iter() {
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

        for node in self.global_nodes.iter_mut() {
            collect(node, &mut ptrs);
        }

        // SAFETY: each pointer is a unique node in the tree; we never alias.
        unsafe { ptrs.into_iter().map(|p| &mut *p).collect() }
    }

    pub fn get_node_with_name(&self, name: &str) -> Option<&Node> {
        self.get_all_nodes()
            .into_iter()
            .find(|node| node.name == name)
    }

    pub fn get_node_with_name_mut(&mut self, name: &str) -> Option<&mut Node> {
        self.get_all_nodes_mut()
            .into_iter()
            .find(|node| node.name == name)
    }

    pub fn get_node(&self, id: u64) -> &Node {
        self.get_all_nodes()
            .into_iter()
            .find(|node| node.id == id)
            .unwrap()
    }

    pub fn get_node_mut(&mut self, id: u64) -> &mut Node {
        self.get_all_nodes_mut()
            .into_iter()
            .find(|node| node.id == id)
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

    pub fn editor_fixed_update(&mut self, delta: f32) {
        let mut systems = inventory::iter::<EditorFixedUpdateSystem>().collect::<Vec<_>>();
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
    pub fn get_node_with_component<T: Component + 'static>(&self) -> Option<&Node> {
        self.get_all_nodes()
            .into_iter()
            .find(|node| node.get_component::<T>().is_some())
    }

    pub fn get_node_with_component_mut<T: Component + 'static>(&mut self) -> Option<&mut Node> {
        self.get_all_nodes_mut()
            .into_iter()
            .find(|node| node.get_component::<T>().is_some())
    }

    pub fn get_global_node_with_component<T: Component + 'static>(&self) -> Option<&Node> {
        self.global_nodes.iter().find(|n| n.has_component::<T>())
    }
    pub fn get_global_node_with_component_mut<T: Component + 'static>(
        &mut self,
    ) -> Option<&mut Node> {
        self.global_nodes
            .iter_mut()
            .find(|n| n.has_component::<T>())
    }

    pub fn serialize_scene(&mut self) -> Result<(), std::io::Error> {
        self.check_node_ids();
        let serialized = SerializedScene {
            root_children: self
                .scene
                .root_node
                .children
                .iter()
                .map(serialize_node)
                .collect(),
            name: self.scene.name.clone(),
            is_primary: self.scene.is_primary,
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

        self.check_node_ids();
        Ok(())
    }

    pub fn new_scene(&mut self) {
        self.scene.name = "Scene".to_string();
        self.scene.root_node.children = Vec::new();
        self.nodes = 0;
    }

    pub fn serialize_scene_not_loaded(&self, scene: &Scene) -> Result<(), std::io::Error> {
        let serialized = SerializedScene {
            root_children: scene
                .root_node
                .children
                .iter()
                .map(serialize_node)
                .collect(),
            name: scene.name.clone(),
            is_primary: scene.is_primary,
        };
        let path = format!("{}/{}.yaml", ENGINE_SCENE_SAVE_PATH, scene.name);
        std::fs::write(path, serde_yaml::to_string(&serialized).unwrap())
    }

    pub fn check_node_names(&mut self) {
        let mut names = Vec::new();

        for node in self.get_all_nodes_mut() {
            let base_name = node.name.clone();

            // check if name already exists
            if names.contains(&base_name) {
                let mut counter = 1;
                let mut new_name = format!("{} ({})", base_name, counter);

                // keep incrementing until it finds an unused name
                while names.contains(&new_name) {
                    counter += 1;
                    new_name = format!("{} ({})", base_name, counter);
                }

                node.name = new_name.clone();
                names.push(new_name);
            } else {
                names.push(base_name);
            }
        }
    }
    pub fn check_node_ids(&mut self) {
        let mut ids = Vec::new();
        let mut next_id = self.nodes;

        let nodes = self.get_all_nodes_mut();

        for node in nodes {
            println!("next_id: {}", next_id);
            if ids.contains(&node.id) {
                node.id = next_id;
                next_id += 1;
            }
            ids.push(node.id);
        }

        self.nodes = next_id;
    }

    fn assign_ids_recursive(&mut self, node: &mut Node) {
        node.id = self.nodes;
        self.nodes += 1;
        for child in node.children.iter_mut() {
            self.assign_ids_recursive(child);
        }
    }

    pub fn add_component_by_name(&mut self, node_id: u64, component_name: &str) -> Result<()> {
        let registration = find_registration(component_name)
            .ok_or_else(|| anyhow::anyhow!("Component '{}' is not registered", component_name))?;

        let component = (registration.create)();

        let node = self
            .get_all_nodes_mut()
            .into_iter()
            .find(|n| n.id == node_id)
            .ok_or_else(|| anyhow::anyhow!("No node with id {} found", node_id))?;

        node.components.push(component);
        Ok(())
    }
}
pub const ENGINE_SCENE_SAVE_PATH: &str = "res/scenes";

#[start]
pub fn start_system(world: &mut World) {
    world.input_manager.deserialize_input_manager().unwrap();
}
