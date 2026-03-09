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
use cgmath::{Rotation, Vector2, Vector3};

use crate::engine::{
    nodes::{
        component::Component,
        components::transform::{ParentGlobal, Transform},
        scene::Scene,
        scene_serialization::{SerializedScene, serialize_node},
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
    /// ```rust
    ///     world.add_node(Node::new());
    ///     world.get_node_mut(0).get_component::<Transform>();
    /// ```

    pub fn get_component<T: Component + 'static>(&self) -> Option<&T> {
        self.components
            .iter()
            .find(|c| c.as_any().type_id() == TypeId::of::<T>())
            .and_then(|c| c.as_any().downcast_ref())
    }

    pub fn get_component_mut<T: Component + 'static>(&mut self) -> Option<&mut T> {
        self.components
            .iter_mut()
            .find(|c| c.as_any().type_id() == TypeId::of::<T>())
            .and_then(|c| c.as_any_mut().downcast_mut())
    }

    pub fn get_component_ptr<T: Component + 'static>(&self) -> Option<*mut T> {
        self.components
            .iter()
            .find(|c| c.as_any().type_id() == TypeId::of::<T>())
            .and_then(|c| {
                let ptr = c.as_ref() as *const dyn Component as *mut dyn Component;
                unsafe { (*ptr).as_any_mut().downcast_mut::<T>().map(|r| r as *mut T) }
            })
    }

    pub fn component_mut<T: Component + 'static>(&self) -> Option<ComponentRef<'_, T>> {
        self.get_component_ptr::<T>().map(|ptr| ComponentRef {
            ptr,
            _marker: std::marker::PhantomData,
        })
    }

    /// Gets mutable components of type (T, T, ...) from the node
    /// ```rust
    ///     world.add_node(Node::new());
    ///     world.get_node_mut(0).get_components_mut::<(&mut Transform, &mut Velocity)>();
    /// ```
    // pub fn get_components_mut<'a, T: ComponentsMut<'a>>(&'a mut self) -> T {
    //     T::from_node(self)
    // }
    pub fn get_components_mut<'a, T: ComponentsMut<'a>>(&'a self) -> T {
        T::from_node(self)
    }
    /// Adds a component of type T to the node
    /// ```rust
    ///     world.add_node(Node::new());
    ///     world.get_node_mut(0).add_component(Transform::default());
    /// ```
    pub fn add_component<T: Component + 'static>(&mut self, component: T) -> &mut Self {
        self.components.push(Box::new(component));
        self
    }

    /// Adds a child to the node
    /// ```rust
    ///     world.add_node(Node::new());
    ///     world.get_node_mut(0).add_child(Node::new());
    /// ```
    pub fn add_child(&mut self, mut child: Node) -> &mut Self {
        child.parent = Some(self.name.clone());
        self.children.push(child);
        self
    }

    /// Propagates the transform of the node to all its children
    /// NOT MANUALLY CALLED
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
            t.calculate_rotation();
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

    /// Adds a component of type T to the node
    /// Note: capitalization is ignored
    /// ```rust
    ///     world.add_node(Node::new());
    ///     world.get_node_mut(0).add_component_by_name("transform");
    /// ```
    pub fn add_component_by_name(&mut self, component_name: &str) -> Result<()> {
        let mut component_name = component_name.to_string();
        component_name = component_name.replace(" ", "");
        component_name = component_name.replace("_", "");

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

pub struct World {
    pub scene: Scene,
    pub scene_manager: SceneManager,
    pub nodes: u64,
    pub global_nodes: Vec<Node>,
    pub input_manager: InputManager,
    pub is_world_hovered: bool,
    pub window_size: Vector2<f32>,
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

impl World {
    pub fn new() -> Self {
        Self {
            scene: Scene::new(".engine/default.scene".to_string()),
            scene_manager: SceneManager::new(),
            nodes: 0,
            global_nodes: Vec::new(),
            input_manager: InputManager::default(),
            is_world_hovered: false,
            window_size: Vector2::new(0.0, 0.0),
        }
    }

    /// Adds a node to the global node list
    /// ```rust
    ///     world.add_global_node(Node::new());
    /// ```
    pub fn add_global_node(&mut self, node: Node) {
        self.global_nodes.push(node);
        self.check_node_ids();
    }
    /// Adds a node to the scene
    /// ```rust
    ///     world.add_node(Node::new());
    /// ```
    pub fn add_node(&mut self, mut node: Node) -> &mut Self {
        self.assign_ids_recursive(&mut node);
        self.scene.root_node.add_child(node);
        self.check_node_names();
        self
    }

    /// Removes a node from the scene
    /// ```rust
    ///     world.remove_node(0);
    /// ```
    pub fn remove_node(&mut self, id: u64) -> &mut Self {
        self.scene.root_node.remove_node(id);
        self.check_node_ids();
        self
    }

    /// Adds a new node to the scene
    /// ```rust
    ///     world.add_new_node();
    /// ```
    pub fn add_new_node(&mut self) {
        self.add_node(Node::new());
        self.check_node_ids();
    }

    /// Gets a reference to all nodes in the world
    /// Note: this excludes the global node
    /// ```rust
    ///     world.add_node(Node::new());
    ///     world.get_node_mut(0).add_component(Transform::default());
    ///     world.get_all_world_nodes();
    /// ```
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

    /// Gets a mutable reference to all nodes in the world
    /// ```rust
    ///     world.add_node(Node::new());
    ///     world.get_node_mut(0).add_component(Transform::default());
    ///     world.get_all_world_nodes_mut();
    /// ```
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

    /// Gets a reference to all nodes in the scene
    /// ```rust
    ///     world.add_node(Node::new());
    ///     world.get_node_mut(0).add_component(Transform::default());
    ///     world.get_all_nodes();
    /// ```
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

    /// Gets a mutable reference to all nodes in the scene
    /// ```rust
    ///     world.add_node(Node::new());
    ///     world.get_node_mut(0).add_component(Transform::default());
    ///     world.get_all_nodes_mut();
    /// ```
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

    /// Gets a reference node with the given name
    /// ```rust
    ///     world.add_node(Node::new());
    ///     world.get_node_mut(0).add_component(Transform::default());
    ///     world.get_node_with_name("Node");
    /// ```
    pub fn get_node_with_name(&self, name: &str) -> Option<&Node> {
        self.get_all_nodes()
            .into_iter()
            .find(|node| node.name == name)
    }

    /// Gets a mutable reference node with the given name
    /// ```rust
    ///     world.add_node(Node::new());
    ///     world.get_node_mut(0).add_component(Transform::default());
    ///     world.get_node_with_name_mut("Node");
    /// ```
    pub fn get_node_with_name_mut(&mut self, name: &str) -> Option<&mut Node> {
        self.get_all_nodes_mut()
            .into_iter()
            .find(|node| node.name == name)
    }

    /// Gets a reference node with the given id
    /// ```rust
    ///     world.add_node(Node::new());
    ///     world.get_node_mut(0).add_component(Transform::default());
    ///     world.get_node(0);
    /// ```
    pub fn get_node(&self, id: u64) -> &Node {
        self.get_all_nodes()
            .into_iter()
            .find(|node| node.id == id)
            .unwrap()
    }

    /// Gets a mutable node with the given id
    /// ```rust
    ///     world.add_node(Node::new());
    ///     world.get_node_mut(0).add_component(Transform::default());
    ///     world.get_node_mut(0);
    /// ```
    pub fn get_node_mut(&mut self, id: u64) -> &mut Node {
        self.get_all_nodes_mut()
            .into_iter()
            .find(|node| node.id == id)
            .unwrap()
    }

    /// Gets the first node with a component of type T
    /// ```rust
    ///     world.add_node(Node::new());
    ///     world.get_node_mut(0).add_component(Transform::default());
    ///     world.get_node_with_component::<Transform>();
    /// ```
    pub fn get_node_with_component<T: Component + 'static>(&self) -> Option<&Node> {
        self.get_all_nodes()
            .into_iter()
            .find(|node| node.get_component::<T>().is_some())
    }

    /// Gets the first node with a component of type T multibly
    /// ```rust
    ///     world.add_node(Node::new());
    ///     world.get_node_mut(0).add_component(Transform::default());
    ///     world.get_node_with_component_mut::<Transform>();
    /// ```
    pub fn get_node_with_component_mut<T: Component + 'static>(&self) -> Option<NodeMut<'_>> {
        fn collect(node: &Node, out: &mut Vec<*mut Node>) {
            out.push(node as *const Node as *mut Node);
            for child in &node.children {
                collect(child, out);
            }
        }

        let mut ptrs: Vec<*mut Node> = Vec::new();
        for node in self.scene.root_node.children.iter() {
            collect(node, &mut ptrs);
        }
        for node in self.global_nodes.iter() {
            collect(node, &mut ptrs);
        }

        unsafe {
            ptrs.into_iter()
                .find(|&p| (*p).has_component::<T>())
                .map(|ptr| NodeMut {
                    ptr,
                    _marker: std::marker::PhantomData,
                })
        }
    }

    /// Gets a component of type T from the global node
    /// ```rust
    ///     world.add_global_node(Node::new());
    ///     world.get_node_mut(0).add_component(Transform::default());
    ///     world.get_global_node_with_component::<Transform>();
    /// ```
    pub fn get_global_node_with_component<T: Component + 'static>(&self) -> Option<&Node> {
        self.global_nodes.iter().find(|n| n.has_component::<T>())
    }

    /// Gets a mutable component of type T from the global node
    /// ```rust
    ///     world.add_global_node(Node::new());
    ///     world.get_node_mut(0).add_component(Transform::default());
    ///     world.get_global_node_with_component_mut::<Transform>();
    /// ```
    pub fn get_global_node_with_component_mut<T: Component + 'static>(
        &mut self,
    ) -> Option<&mut Node> {
        self.global_nodes
            .iter_mut()
            .find(|n| n.has_component::<T>())
    }

    /// Adds a component of type T to the node with the given id
    /// Note: capitalization is ignored
    /// ```rust
    ///     world.add_node(Node::new());
    ///     world.get_node_mut(0).add_component_by_name(0, "transform");
    /// ```
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

    /// Creates a new scene.
    /// ```rust
    ///     world.new_scene();
    /// ```
    pub fn new_scene(&mut self) {
        self.scene.name = "Scene".to_string();
        self.scene.root_node.children = Vec::new();
        self.nodes = 0;
    }

    /// Serializes the current scene to a file
    /// ```rust
    ///     world.serialize_scene();
    /// ```
    pub fn serialize_scene(&mut self) -> Result<(), std::io::Error> {
        self.check_node_ids();
        let path = self.scene.path.clone();
        let serialized = SerializedScene {
            root_children: self
                .scene
                .root_node
                .children
                .iter()
                .map(serialize_node)
                .collect(),
            path: path.clone(),
            name: self.scene.name.clone(),
            is_primary: self.scene.is_primary,
        };
        std::fs::write(path, serde_yaml::to_string(&serialized).unwrap())
    }

    /// Deserializes a scene from a file
    pub fn deserialize_scene(&mut self, _scene: String) -> Result<(), serde_yaml::Error> {
        // let path = format!("{}/{}.yaml", ASSET_DIR, scene);
        // let contents = std::fs::read_to_string(&path).expect("Failed to read scene file");
        // let serialized: SerializedScene = serde_yaml::from_str(&contents)?;
        // self.scene.root_node.children = serialized
        //     .root_children
        //     .into_iter()
        //     .map(deserialize_node)
        //     .collect();
        //
        // self.check_node_ids();
        Ok(())
    }

    /// Serializes a scene that isn't loaded into the engine.
    pub fn serialize_scene_not_loaded(&self, _scene: &Scene) -> Result<(), std::io::Error> {
        // let serialized = SerializedScene {
        //     root_children: scene
        //         .root_node
        //         .children
        //         .iter()
        //         .map(serialize_node)
        //         .collect(),
        //     name: scene.name.clone(),
        //     is_primary: scene.is_primary,
        //     asset_path: scene.path.clone(),
        // };
        // let path = format!("{}/{}.yaml", ASSET_DIR, scene.name);
        // std::fs::write(path, serde_yaml::to_string(&serialized).unwrap())
        Ok(())
    }

    /// Checks that all node names are unique
    /// TODO: find out if this is necessary
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

    // Checks that all nodes have unique ids
    // NOTE: this is called automatically when adding a node
    pub fn check_node_ids(&mut self) {
        let mut next_id = 0u64;
        for node in self.get_all_nodes_mut() {
            node.id = next_id;
            next_id += 1;
        }
        self.nodes = next_id;
    }

    /// Assigns ids to all nodes in the tree
    /// NOTE: this is called automatically when adding a node
    fn assign_ids_recursive(&mut self, node: &mut Node) {
        node.id = self.nodes;
        self.nodes += 1;
        for child in node.children.iter_mut() {
            self.assign_ids_recursive(child);
        }
    }

    /// Runs all start systems
    pub fn start(&mut self) {
        let mut systems = inventory::iter::<StartSystem>().collect::<Vec<_>>();
        systems.sort_by(|a, b| a.priority.cmp(&b.priority));
        systems.reverse();
        for system in systems.iter_mut() {
            (system.func)(self);
        }
    }

    /// Runs all update systems
    pub fn update(&mut self) {
        let mut systems = inventory::iter::<UpdateSystem>().collect::<Vec<_>>();
        systems.sort_by(|a, b| a.priority.cmp(&b.priority));
        systems.reverse();
        for system in systems.iter_mut() {
            (system.func)(self);
        }
    }

    /// Runs all fixed update systems
    pub fn fixed_update(&mut self, delta: f32) {
        let mut systems = inventory::iter::<FixedUpdateSystem>().collect::<Vec<_>>();
        systems.sort_by(|a, b| a.priority.cmp(&b.priority));
        systems.reverse();
        for system in systems.iter_mut() {
            (system.func)(self, delta);
        }
    }

    /// Runs all editor fixed update systems
    pub fn editor_fixed_update(&mut self, delta: f32) {
        let mut systems = inventory::iter::<EditorFixedUpdateSystem>().collect::<Vec<_>>();
        systems.sort_by(|a, b| a.priority.cmp(&b.priority));
        systems.reverse();
        for system in systems.iter_mut() {
            (system.func)(self, delta);
        }
    }

    /// Runs all late update systems
    pub fn late_update(&mut self) {
        let mut systems = inventory::iter::<LateUpdateSystem>().collect::<Vec<_>>();
        systems.sort_by(|a, b| a.priority.cmp(&b.priority));
        systems.reverse();
        for system in systems.iter_mut() {
            (system.func)(self);
        }
    }
}

/// Trait for getting mutable references to a single node
pub struct NodeMut<'a> {
    ptr: *mut Node,
    _marker: std::marker::PhantomData<&'a mut Node>,
}

impl<'a> std::ops::Deref for NodeMut<'a> {
    type Target = Node;
    fn deref(&self) -> &Node {
        unsafe { &*self.ptr }
    }
}

impl<'a> std::ops::DerefMut for NodeMut<'a> {
    fn deref_mut(&mut self) -> &mut Node {
        unsafe { &mut *self.ptr }
    }
}

/// Trait for getting references to a single component from one node
pub struct ComponentRef<'a, T> {
    ptr: *mut T,
    _marker: std::marker::PhantomData<&'a mut T>,
}

impl<'a, T> std::ops::Deref for ComponentRef<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.ptr }
    }
}

impl<'a, T> std::ops::DerefMut for ComponentRef<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.ptr }
    }
}

/// Trait for getting mutable references to multiple components from one node
pub trait ComponentsMut<'a> {
    fn from_node(node: &'a Node) -> Self;
}

macro_rules! impl_components_mut {
    ($($T:ident),+) => {
        #[allow(nonstandard_style)]
        impl<'a, $($T: Component + 'static),+> ComponentsMut<'a> for ($(&'a mut $T),+) {
            fn from_node(node: &'a Node) -> Self {
                $(let mut $T: Option<*mut $T> = None;)+

                for component in node.components.iter() {
                    let ptr = component.as_ref() as *const dyn Component as *mut dyn Component;
                    let any = unsafe { (*ptr).as_any_mut() };
                    let type_id = (*component).as_any().type_id();
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
                            .unwrap_or_else(|| panic!("Component ({}) not found on node", std::any::type_name::<$T>()))
                    ),+)
                }
            }
        }
    };
}

impl_components_mut!(A, B);
impl_components_mut!(A, B, C);
impl_components_mut!(A, B, C, D);
impl_components_mut!(A, B, C, D, E);
impl_components_mut!(A, B, C, D, E, F);
impl_components_mut!(A, B, C, D, E, F, G);
impl_components_mut!(A, B, C, D, E, F, G, H);

#[start]
pub fn start_system(world: &mut World) {
    world.input_manager.deserialize_input_manager().unwrap();
}
