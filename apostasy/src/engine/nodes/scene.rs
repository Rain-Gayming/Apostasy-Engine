use std::io::Error;

use crate::{
    self as apostasy,
    engine::nodes::{
        Component, Node,
        components::{light::Light, skybox::Skybox, transform::Transform},
    },
    log, log_warn,
};
use apostasy_macros::{Component, InspectValue, Inspectable, SerializableComponent};
use serde::{Deserialize, Serialize};

use crate::engine::nodes::scene_serialization::{
    SerializedScene, deserialize_node, parse_root_children_from_value, serialize_node,
};

pub struct Scene {
    pub name: String,
    pub path: String,
    pub root_node: Box<Node>,
    pub is_primary: bool,
}

impl Default for Scene {
    fn default() -> Self {
        Self::new("new_scene".to_string())
    }
}

impl Scene {
    pub fn new(path: String) -> Self {
        let mut root_node = Node::new();
        root_node.name = "root".to_string();

        add_default_nodes(&mut root_node);

        Self {
            name: "Scene".to_string(),
            path,
            root_node: Box::new(root_node),
            is_primary: false,
        }
    }
}

/// Setups the default world environment
/// deletes the current environmnet
pub fn add_default_nodes(node: &mut Node) {
    let mut skybox = Node::new();
    skybox.name = "Skybox".to_string();
    skybox.add_component(Skybox::default());
    skybox.add_component(Transform::default());
    node.add_child(skybox);

    let mut light = Node::new();
    light.name = "Directional Light".to_string();
    light.add_component(Light::default());
    let mut transform = Transform::default();
    transform.position.y = 10.0;
    light.add_component(transform);
    node.add_child(light);
}

#[derive(
    Clone,
    Serialize,
    Deserialize,
    Debug,
    Component,
    InspectValue,
    Inspectable,
    SerializableComponent,
    Default,
)]
pub struct SceneInstance {
    /// Path to the source scene file this node was instanced from
    pub source_path: String,
    /// Whether this instance is "unpacked" (edits are local, no longer linked)
    pub unpacked: bool,
}

impl SceneInstance {
    pub fn new(source_path: impl Into<String>) -> Self {
        Self {
            source_path: source_path.into(),
            unpacked: false,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct SceneManager {
    #[serde(skip)]
    pub scenes: Vec<Scene>,

    pub scene_paths: Vec<String>,
    pub primary_scene: Option<String>,
}

impl Default for SceneManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SceneManager {
    pub fn new() -> Self {
        let scene_manager = Self {
            scenes: Vec::new(),
            scene_paths: Vec::new(),
            primary_scene: None,
        };
        // scene_manager.load_scenes();
        scene_manager
    }

    pub fn load_scene(&mut self, name: &str) -> Option<Scene> {
        deserialize_scene(name.to_string())
    }

    pub fn remove_scene(&mut self, path: &str) {
        self.scenes.retain(|s| s.path != path);
        self.scene_paths.retain(|s| s != path);
    }

    pub fn set_scene_primary(&mut self, name: &str, is_primary: bool) {
        for scene in self.scenes.iter_mut() {
            scene.is_primary = false;
        }

        let scene = self.scenes.iter_mut().find(|s| s.name == name).unwrap();
        scene.is_primary = is_primary;
    }

    pub fn get_primary_scene(&mut self) {
        if let Some(primary_scene) = self.scenes.iter().find(|s| s.is_primary) {
            self.primary_scene = Some(primary_scene.name.clone());
        } else {
            log_warn!("No existing priamry scene, add one via the Editor Settings");
        }
    }

    pub fn serialize_scene_manager(&mut self) -> Result<(), Error> {
        std::fs::write(
            "res/scene_manager.yaml",
            serde_yaml::to_string(&self).unwrap(),
        )
    }
}

pub fn deserialize_scene_manager() -> Option<SceneManager> {
    let contents = match std::fs::read_to_string("res/scene_manager.yaml") {
        Ok(c) => c,
        Err(err) => {
            log_warn!(
                "Failed to read scene manager file {}: {}",
                "res/scene_manager.yaml",
                err
            );
            return None;
        }
    };

    let value: serde_yaml::Value = match serde_yaml::from_str(&contents) {
        Ok(v) => v,
        Err(err) => {
            eprintln!(
                "Failed to parse scene manager YAML {}: {}",
                "res/scene_manager.yaml", err
            );
            return None;
        }
    };

    let mut scene_manager = SceneManager::new();

    // scene_paths
    let raw_paths: serde_yaml::Value = value.get("scene_paths").unwrap().clone();
    let paths = raw_paths.as_sequence().unwrap();

    for path in paths {
        let path = path.as_str().unwrap().to_string();

        let scene = scene_manager.load_scene(&path);
        if let Some(scene) = scene {
            scene_manager.scenes.push(scene);
        } else {
            log_warn!("Failed to load scene: {}", path);
        }

        scene_manager.scene_paths.push(path);
    }

    // primary_scene
    scene_manager.primary_scene = value
        .get("primary_scene")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    Some(scene_manager)
}

pub fn deserialize_scene(path: String) -> Option<Scene> {
    let contents = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(err) => {
            eprintln!("Failed to read scene file {}: {}", path, err);
            return Some(Scene::new(path));
        }
    };

    let value: serde_yaml::Value = match serde_yaml::from_str(&contents) {
        Ok(v) => v,
        Err(err) => {
            eprintln!("Failed to parse scene YAML {}: {}", path, err);
            return Some(Scene::new(path));
        }
    };

    let mut scene = Scene::new(path.clone());

    // root_children
    if let Some(root_children_value) = value.get("root_children") {
        let parsed = parse_root_children_from_value(root_children_value);
        scene.root_node.children = parsed.into_iter().map(deserialize_node).collect();
    }

    // name
    if let Some(n) = value.get("name").and_then(|v| v.as_str()) {
        scene.name = n.to_string();
    }

    // is_primary
    if let Some(p) = value.get("is_primary").and_then(|v| v.as_bool()) {
        scene.is_primary = p;
    }

    Some(scene)
}

pub fn serialize_scene(scene: Scene) -> Result<(), Error> {
    let path = scene.path.clone();
    let serialized = SerializedScene {
        root_children: scene
            .root_node
            .children
            .iter()
            .map(serialize_node)
            .collect(),
        path: path.clone(),
        name: scene.name.clone(),
        is_primary: scene.is_primary,
    };
    std::fs::write(path, serde_yaml::to_string(&serialized).unwrap())
}

pub fn instance_scene_as_node(name: &str, path: &str) -> Node {
    let mut root = Node::new();
    root.name = name.to_string();
    root.add_component(SceneInstance::new(path));

    if let Some(source_scene) = deserialize_scene(path.to_string()) {
        root.children = source_scene.root_node.children;
    }
    root
}
