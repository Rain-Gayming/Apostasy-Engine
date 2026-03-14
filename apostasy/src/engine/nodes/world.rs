use anyhow::Result;
use cgmath::Vector2;

use crate::engine::{
    nodes::{
        Node, NodeMut, build_instance_node,
        component::Component,
        scene::{Scene, SceneInstance, SceneManager, deserialize_scene},
        scene_serialization::{SerializedScene, find_registration, serialize_node},
        system::{
            EditorFixedUpdateSystem, FixedUpdateSystem, LateUpdateSystem, StartSystem, UpdateSystem,
        },
    },
    windowing::input_manager::InputManager,
};

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
        if !node.exempt_from_id_check {
            self.assign_ids_recursive(&mut node);
        }
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

    /// Instance a scene file into the current scene.
    /// Drag a .scene asset onto the viewport or scene tree to call this.
    pub fn instance_scene(&mut self, path: &str) -> &mut Self {
        let node = build_instance_node(path);
        self.add_node(node);
        self
    }

    /// Instance a scene as a child of an existing node.
    pub fn instance_scene_under(&mut self, parent_id: u64, path: &str) {
        let node = build_instance_node(path);
        self.scene.root_node.insert_under(parent_id, node);
        self.check_node_ids();
    }

    /// Break the link to the source scene — the node becomes standalone.
    pub fn unpack_scene_instance(&mut self, node_id: u64) {
        let node = self.get_node_mut(node_id);
        if let Some(instance) = node.get_component_mut::<SceneInstance>() {
            instance.unpacked = true;
        }
    }

    /// Reload all live (non-unpacked) scene instances from their source files.
    /// Call this whenever a source scene is saved in the editor.
    pub fn reload_scene_instances(&mut self) {
        let stubs: Vec<(u64, String)> = self
            .get_all_world_nodes()
            .iter()
            .filter_map(|n| {
                n.get_component::<SceneInstance>()
                    .filter(|i| !i.unpacked)
                    .map(|i| (n.id, i.source_path.clone()))
            })
            .collect();

        for (id, path) in stubs {
            if let Some(source) = deserialize_scene(path) {
                self.get_node_mut(id).children = source.root_node.children;
            }
        }
        self.check_node_ids();
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
    pub fn deserialize_scene(&mut self, scene: String) -> Result<(), serde_yaml::Error> {
        if let Some(loaded) = deserialize_scene(scene) {
            self.scene = loaded;
            self.check_node_ids();
        }
        Ok(())
    }

    /// Serializes a scene that isn't loaded into the engine.
    pub fn serialize_scene_not_loaded(&self, scene: &Scene) -> Result<(), std::io::Error> {
        let path = scene.path.clone();
        let serialized = SerializedScene {
            root_children: scene
                .root_node
                .children
                .iter()
                .map(serialize_node)
                .collect(),
            name: scene.name.clone(),
            is_primary: scene.is_primary,
            path: scene.path.clone(),
        };
        std::fs::write(path, serde_yaml::to_string(&serialized).unwrap())
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
            if !node.exempt_from_id_check {
                node.id = next_id;
            }
            next_id += 1;
        }
        self.nodes = next_id;
    }

    /// Assigns ids to all nodes in the tree
    /// NOTE: this is called automatically when adding a node
    fn assign_ids_recursive(&mut self, node: &mut Node) {
        node.id = self.nodes;
        for child in node.children.iter_mut() {
            if !child.exempt_from_id_check {
                self.assign_ids_recursive(child);
            }
            self.nodes += 1;
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
