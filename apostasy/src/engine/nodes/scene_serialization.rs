use serde::{Deserialize, Serialize};
use serde_yaml;

use super::Node;
use crate::engine::{assets::AssetPath, nodes::component::Component};
use serde_yaml::Value;

#[derive(Serialize, Deserialize)]
/// A serialized component, contains the type name and data
pub struct SerializedComponent {
    #[serde(rename = "type")]
    type_name: String,
    data: serde_yaml::Value,
}

#[derive(Serialize, Deserialize)]
/// A serialized node, contains a list of components and children
pub struct SerializedNode {
    name: String,
    id: u64,
    components: Vec<SerializedComponent>,
    children: Vec<SerializedNode>,
}

#[derive(Serialize, Deserialize)]
/// A serialized scene, contains a list of root children
pub struct SerializedScene {
    pub root_children: Vec<SerializedNode>,
    pub name: String,
    pub is_primary: bool,
    pub asset_path: AssetPath,
}

/// A component registrator, used to serialize and deserialize components
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

/// Serializes a node, returns a serialized node
pub fn serialize_node(node: &Node) -> SerializedNode {
    let _: Vec<SerializedComponent> = node
        .components
        .iter()
        .filter_map(|component| {
            let type_name = component.type_name();

            let registration = find_registration(type_name);

            let registration = registration?;
            Some(SerializedComponent {
                type_name: type_name.to_string(),
                data: (registration.serialize)(component.as_ref()),
            })
        })
        .collect();
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

    SerializedNode {
        name: node.name.clone(),
        id: node.id,
        components,
        children: node.children.iter().map(serialize_node).collect(),
    }
}

/// Deserializes a serialized node, returns a node
pub fn deserialize_node(serialized: SerializedNode) -> Node {
    let components: Vec<Box<dyn Component>> = serialized
        .components
        .into_iter()
        .filter_map(|sc| {
            let registration = find_registration(&sc.type_name)?;
            Some((registration.deserialize)(sc.data))
        })
        .collect();

    Node {
        name: serialized.name.clone(),
        id: serialized.id,
        editing_name: serialized.name,
        children: serialized
            .children
            .into_iter()
            .map(deserialize_node)
            .collect(),
        parent: None,
        components,
    }
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

    // components
    let components = mapping
        .get(&Value::String("components".to_string()))
        .and_then(|v| match v {
            Sequence(seq) => Some(
                seq.iter()
                    .filter_map(|entry| {
                        // each component should be a mapping with "type" and "data"
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

    // children
    let children = mapping
        .get(&Value::String("children".to_string()))
        .and_then(|v| match v {
            Sequence(seq) => Some(
                seq.iter()
                    .filter_map(|entry| parse_serialized_node(entry))
                    .collect(),
            ),
            _ => None,
        })
        .unwrap_or_default();

    Some(SerializedNode {
        name,
        id,
        components,
        children,
    })
}

/// Parse a root sequence of SerializedNodes from a YAML value safely.
pub fn parse_root_children_from_value(value: &Value) -> Vec<SerializedNode> {
    if let Value::Sequence(seq) = value {
        seq.iter()
            .filter_map(|v| parse_serialized_node(v))
            .collect()
    } else {
        Vec::new()
    }
}
