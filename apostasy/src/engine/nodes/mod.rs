use crate::{
    self as apostasy,
    engine::nodes::{
        scene::{SceneInstance, deserialize_scene},
        scene_serialization::find_registration,
        world::World,
    },
    log, log_warn,
};
use std::any::TypeId;

use anyhow::Result;
use apostasy_macros::start;
use cgmath::{Rotation, Vector3};

use crate::engine::nodes::{
    component::Component,
    components::transform::{ParentGlobal, Transform},
};

pub mod component;
pub mod components;
pub mod scene;
pub mod scene_serialization;
pub mod system;
pub mod world;

#[derive(Clone)]
pub struct Node {
    pub name: String,
    pub id: u64,
    pub editing_name: String,
    pub children: Vec<Node>,
    pub parent: Option<u64>,
    pub components: Vec<Box<dyn Component>>,
    pub exempt_from_id_check: bool,
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
            exempt_from_id_check: false,
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
            .any(|component| component.as_any().downcast_ref::<T>().is_some())
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
        if self.get_component::<T>().is_some() {
            log!("You can only have one of any component on an entity");
            return self;
        } else {
            self.components.push(Box::new(component));
            return self;
        }
    }

    /// Adds a child to the node
    /// ```rust
    ///     world.add_node(Node::new());
    ///     world.get_node_mut(0).add_child(Node::new());
    /// ```
    pub fn add_child(&mut self, mut child: Node) -> &mut Self {
        child.parent = Some(self.id.clone());
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
            node.parent = Some(self.id.clone());
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

        // Check for duplicate using type_name since we don't have T here
        let component = (registration.create)();
        let new_type_name = component.type_name();

        if self
            .components
            .iter()
            .any(|c| c.type_name() == new_type_name)
        {
            log!("You can only have one of any component on an entity");
            return Ok(());
        }

        self.components.push(component);
        Ok(())
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

fn build_instance_node(path: &str) -> Node {
    let node_name = std::path::Path::new(path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("SceneInstance")
        .to_string();

    let mut node = Node::new();
    node.name = node_name;
    node.add_component(SceneInstance::new(path));

    // Eagerly load children so the instance is immediately usable in the editor
    if let Some(source) = deserialize_scene(path.to_string()) {
        node.children = source.root_node.children;
    }

    node
}
