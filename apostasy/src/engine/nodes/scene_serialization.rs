use serde::{Deserialize, Serialize};
use serde_yaml;

use super::Node;
use crate::engine::nodes::component::Component;

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
