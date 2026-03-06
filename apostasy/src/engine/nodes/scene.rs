use std::path::Path;

use crate::engine::nodes::{
    Node,
    scene_serialization::{SerializedScene, deserialize_node, serialize_node},
};

pub struct Scene {
    pub name: String,
    pub root_node: Box<Node>,
    pub is_primary: bool,
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
            is_primary: false,
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
        let mut scene_manager = Self {
            scenes: Vec::new(),
            primary_scene: None,
        };
        scene_manager.load_scenes();
        scene_manager
    }

    pub fn load_scenes(&mut self) {
        // let scenes = std::fs::read_dir(ASSET_DIR).unwrap();
        // for scene in scenes {
        //     let scene = scene.unwrap();
        //
        //     if scene.file_type().unwrap().is_dir() {
        //         continue;
        //     }
        //
        //     let name = scene.file_name().into_string().unwrap();
        //     if !name.ends_with(".yaml") {
        //         continue;
        //     }
        //     let name = name
        //         .strip_suffix(".yaml")
        //         .expect("Scene file isnt yaml")
        //         .to_string();
        //
        //     let scene = self.deserialize_scene(name);
        //     if let Some(scene) = scene {
        //         self.scenes.push(scene);
        //     }
        // }
    }

    pub fn serialize_scenes(&mut self) {
        for scene in self.scenes.iter_mut() {
            println!("Serializing scene: {}", scene.name);
            // let path = format!("{}/{}.yaml", ASSET_DIR, scene.name);
            // if !Path::new(ASSET_DIR).exists() {
            //     let _ = std::fs::create_dir_all(ASSET_DIR);
            // }
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
            // let _ = std::fs::write(path, serde_yaml::to_string(&serialized).unwrap());
        }
    }

    pub fn deserialize_scene(&mut self, scene: String) -> Option<Scene> {
        // let path = format!("{}/{}.yaml", ASSET_DIR, scene);
        //
        // let asset_path = match std::fs::read_to_string(&path) {
        //     Ok(c) => AssetPath::new(path.clone(), scene, "yaml".to_string(), AssetType::Scene),
        //     Err(err) => {
        //         eprintln!("Failed to read scene file {}: {}", path, err);
        //         return Some(Scene::new());
        //     }
        // };
        //
        //
        // let contents = match std::fs::read_to_string(&path) {
        //     Ok(c) => c,
        //     Err(err) => {
        //         eprintln!("Failed to read scene file {}: {}", path, err);
        //         return Some(Scene::new());
        //     }
        // };
        //
        // let value: serde_yaml::Value = match serde_yaml::from_str(&contents) {
        //     Ok(v) => v,
        //     Err(err) => {
        //         eprintln!("Failed to parse scene YAML {}: {}", path, err);
        //         return Some(Scene::new());
        //     }
        // };
        //
        // let mut scene = Scene::new();
        //
        // // root_children
        // if let Some(root_children_value) = value.get("root_children") {
        //     let parsed = crate::engine::nodes::scene_serialization::parse_root_children_from_value(
        //         root_children_value,
        //     );
        //     scene.root_node.children = parsed.into_iter().map(deserialize_node).collect();
        // }
        //
        // // name
        // if let Some(n) = value.get("name").and_then(|v| v.as_str()) {
        //     scene.name = n.to_string();
        // }
        //
        // // is_primary
        // if let Some(p) = value.get("is_primary").and_then(|v| v.as_bool()) {
        //     scene.is_primary = p;
        // }
        //
        let scene = Scene::new();

        Some(scene)
    }

    pub fn load_scene(&mut self, name: &str) -> Option<Scene> {
        if let Some(_) = self.deserialize_scene(name.to_string()) {
            self.deserialize_scene(name.to_string())
        } else {
            None
        }
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
