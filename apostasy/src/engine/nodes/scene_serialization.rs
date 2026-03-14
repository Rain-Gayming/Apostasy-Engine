use serde::{Deserialize, Serialize};
use serde_yaml;

use super::Node;
use crate::{
    engine::{
        assets::asset::{Asset, AssetLoadError, AssetLoader},
        nodes::{component::Component, scene::SceneInstance},
    },
    log,
};
use serde_yaml::Value;

#[derive(Serialize, Deserialize)]
pub struct SerializedComponent {
    #[serde(rename = "type")]
    type_name: String,
    data: serde_yaml::Value,
}

#[derive(Serialize, Deserialize)]
pub struct SerializedNode {
    name: String,
    id: u64,
    components: Vec<SerializedComponent>,
    children: Vec<SerializedNode>,
    parent: Option<u64>,
    pub scene_instance_path: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct SerializedScene {
    pub root_children: Vec<SerializedNode>,
    pub name: String,
    pub path: String,
    pub is_primary: bool,
}

impl Asset for SerializedScene {
    fn asset_type_name() -> &'static str {
        "Scene"
    }
}

pub struct SceneLoader;

impl AssetLoader for SceneLoader {
    type Asset = SerializedScene;
    fn extensions(&self) -> &[&str] {
        &["scene"]
    }

    fn load_sync(&self, path: &std::path::Path) -> Result<SerializedScene, AssetLoadError> {
        let src = std::fs::read_to_string(path).map_err(|e| AssetLoadError::Io {
            path: path.display().to_string(),
            source: e,
        })?;

        let mut scene: SerializedScene =
            serde_yaml::from_str(&src).map_err(|e| AssetLoadError::Parse {
                path: path.display().to_string(),
                message: e.to_string(),
            })?;

        scene.path = path.display().to_string();
        Ok(scene)
    }
}

pub struct ComponentRegistrator {
    pub type_name: &'static str,
    pub serialize: fn(&dyn Component) -> serde_yaml::Value,
    pub deserialize: fn(serde_yaml::Value) -> Box<dyn Component>,
    pub create: fn() -> Box<dyn Component>,
}

inventory::collect!(ComponentRegistrator);

pub fn find_registration(type_name: &str) -> Option<&'static ComponentRegistrator> {
    inventory::iter::<ComponentRegistrator>()
        .find(|r| r.type_name.to_lowercase() == type_name.to_lowercase())
}

pub fn serialize_node(node: &Node) -> SerializedNode {
    if let Some(instance) = node.get_component::<SceneInstance>() {
        if !instance.unpacked {
            let serialized_components: Vec<_> = node
                .components
                .iter()
                .filter_map(|c| {
                    let type_name = c.type_name();
                    let reg = find_registration(type_name);

                    let reg = reg?;
                    Some(SerializedComponent {
                        type_name: type_name.to_string(),
                        data: (reg.serialize)(c.as_ref()),
                    })
                })
                .collect();

            let mut parent = None;
            if let Some(parent_id) = node.parent {
                parent = Some(parent_id);
            }
            return SerializedNode {
                name: node.name.clone(),
                id: node.id,
                components: serialized_components,
                children: vec![],
                parent,
                scene_instance_path: Some(instance.source_path.clone()),
            };
        }
    }

    let components = node
        .components
        .iter()
        .filter_map(|component| {
            let type_name = component.type_name();
            let registration = find_registration(type_name)?;
            Some(SerializedComponent {
                type_name: type_name.to_string(),
                data: (registration.serialize)(component.as_ref()),
            })
        })
        .collect();

    let mut parent = None;
    if let Some(parent_id) = node.parent {
        parent = Some(parent_id);
    }
    SerializedNode {
        name: node.name.clone(),
        id: node.id,
        components,
        parent,
        children: node.children.iter().map(serialize_node).collect(),
        scene_instance_path: None,
    }
}

pub fn deserialize_node(serialized: SerializedNode) -> Node {
    let components: Vec<Box<dyn Component>> = serialized
        .components
        .into_iter()
        .filter_map(|sc| {
            let registration = find_registration(&sc.type_name)?;
            Some((registration.deserialize)(sc.data))
        })
        .collect();

    let children = if let Some(ref path) = serialized.scene_instance_path {
        load_instance_children(path)
    } else {
        serialized
            .children
            .into_iter()
            .map(deserialize_node)
            .collect()
    };

    Node {
        name: serialized.name.clone(),
        id: serialized.id,
        editing_name: serialized.name,
        children,
        parent: None,
        components,
        exempt_from_id_check: false,
    }
}

fn load_instance_children(path: &str) -> Vec<Node> {
    let contents = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            log!(
                "[scene_instance] Failed to read source scene '{}': {}",
                path,
                e
            );
            return vec![];
        }
    };

    let value: serde_yaml::Value = match serde_yaml::from_str(&contents) {
        Ok(v) => v,
        Err(e) => {
            eprintln!(
                "[scene_instance] Failed to parse source scene '{}': {}",
                path, e
            );
            return vec![];
        }
    };

    value
        .get("root_children")
        .map(|v| {
            parse_root_children_from_value(v)
                .into_iter()
                .map(deserialize_node)
                .collect()
        })
        .unwrap_or_default()
}

fn parse_serialized_node(value: &Value) -> Option<SerializedNode> {
    use serde_yaml::Value::{Mapping, Sequence};

    let mapping = match value {
        Mapping(m) => m,
        _ => return None,
    };

    let get_str = |k: &str| -> Option<String> {
        mapping
            .get(&Value::String(k.to_string()))
            .and_then(|v| v.as_str().map(|s| s.to_string()))
    };

    let get_u64 = |k: &str| -> Option<u64> {
        mapping
            .get(&Value::String(k.to_string()))
            .and_then(|v| v.as_u64())
    };

    let name = get_str("name").unwrap_or_else(|| "node".to_string());
    let id = get_u64("id").unwrap_or(0);

    // scene_instance_path — if present this node is an instance stub
    let scene_instance_path = get_str("scene_instance_path");

    let components = mapping
        .get(&Value::String("components".to_string()))
        .and_then(|v| match v {
            Sequence(seq) => Some(
                seq.iter()
                    .filter_map(|entry| {
                        if let Value::Mapping(cm) = entry {
                            let type_val = cm.get(&Value::String("type".to_string()));
                            let data_val = cm.get(&Value::String("data".to_string()));
                            if let Some(Value::String(t)) = type_val {
                                let data = data_val.cloned().unwrap_or(Value::Null);
                                return Some(SerializedComponent {
                                    type_name: t.clone(),
                                    data,
                                });
                            }
                        }
                        None
                    })
                    .collect(),
            ),
            _ => None,
        })
        .unwrap_or_default();

    let children = if scene_instance_path.is_some() {
        vec![]
    } else {
        mapping
            .get(&Value::String("children".to_string()))
            .and_then(|v| match v {
                Sequence(seq) => Some(
                    seq.iter()
                        .filter_map(|entry| parse_serialized_node(entry))
                        .collect(),
                ),
                _ => None,
            })
            .unwrap_or_default()
    };

    let parent = get_u64("parent");
    Some(SerializedNode {
        name,
        id,
        components,
        children,
        parent,
        scene_instance_path,
    })
}

pub fn parse_root_children_from_value(value: &Value) -> Vec<SerializedNode> {
    if let Value::Sequence(seq) = value {
        seq.iter()
            .filter_map(|v| parse_serialized_node(v))
            .collect()
    } else {
        Vec::new()
    }
}
