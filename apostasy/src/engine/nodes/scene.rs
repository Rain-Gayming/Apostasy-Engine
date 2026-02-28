use crate::engine::nodes::{
    ENGINE_SCENE_SAVE_PATH, Node,
    scene_serialization::{SerializedScene, deserialize_node},
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
        let scenes = std::fs::read_dir(ENGINE_SCENE_SAVE_PATH).unwrap();
        for scene in scenes {
            let scene = scene.unwrap();
            let name = scene.file_name().into_string().unwrap();
            let name = name.strip_suffix(".yaml").unwrap().to_string();

            let scene = self.deserialize_scene(name);
            self.scenes.push(scene);
        }
    }

    pub fn deserialize_scene(&mut self, scene: String) -> Scene {
        let path = format!("{}/{}.yaml", ENGINE_SCENE_SAVE_PATH, scene);
        let contents = std::fs::read_to_string(&path).expect("Failed to read scene file");
        let serialized: SerializedScene = serde_yaml::from_str(&contents).unwrap();
        let mut scene = Scene::new();
        scene.root_node.children = serialized
            .root_children
            .into_iter()
            .map(deserialize_node)
            .collect();
        scene.name = serialized.name;
        scene.is_primary = serialized.is_primary;

        scene
    }

    pub fn load_scene(&mut self, name: &str) -> Scene {
        self.deserialize_scene(name.to_string())
    }

    pub fn remove_scene(&mut self, name: &str) {
        self.scenes.retain(|scene| scene.name != name);
        std::fs::remove_file(format!("{}/{}.yaml", ENGINE_SCENE_SAVE_PATH, name)).unwrap();
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
