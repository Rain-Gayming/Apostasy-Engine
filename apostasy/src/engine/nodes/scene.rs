use crate::{
    self as apostasy,
    engine::nodes::{Component, Node},
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
        Self {
            name: "Scene".to_string(),
            path,
            root_node: Box::new(root_node),
            is_primary: false,
        }
    }
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

pub struct SceneManager {
    pub scenes: Vec<Scene>,
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
            primary_scene: None,
        };
        // scene_manager.load_scenes();
        scene_manager
    }

    pub fn load_scene(&mut self, name: &str) -> Option<Scene> {
        deserialize_scene(name.to_string())
    }

    pub fn remove_scene(&mut self, path: &str) {
        // self.scenes.retain(|scene| scene.name != name);
        std::fs::remove_file(path).unwrap();
    }

    pub fn set_scene_primary(&mut self, name: &str, is_primary: bool) {
        for scene in self.scenes.iter_mut() {
            scene.is_primary = false;
        }

        let scene = self.scenes.iter_mut().find(|s| s.name == name).unwrap();
        scene.is_primary = is_primary;
    }

    pub fn get_primary_scene(&mut self) {
        self.primary_scene = Some(
            self.scenes
                .iter()
                .find(|s| s.is_primary)
                .unwrap()
                .name
                .clone(),
        );
    }
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

pub fn serialize_scene(scene: Scene) -> Result<(), std::io::Error> {
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
