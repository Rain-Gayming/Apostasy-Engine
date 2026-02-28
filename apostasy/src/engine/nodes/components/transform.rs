use crate::engine::editor::inspectable::{InspectValue, Inspectable};
use crate::engine::nodes::World;
use crate::{self as apostasy};
use apostasy_macros::{Component, Inspectable, SerializableComponent, update};
use cgmath::{Deg, Euler, One, Quaternion, Rotation, Vector3};
use serde::{Deserialize, Serialize};

#[derive(Component, Clone, Inspectable, Serialize, Deserialize, SerializableComponent)]
pub struct Transform {
    pub position: Vector3<f32>,
    pub rotation: Quaternion<f32>,
    pub rotation_euler: Vector3<f32>,
    pub scale: Vector3<f32>,
    up: Vector3<f32>,
    forward: Vector3<f32>,
    right: Vector3<f32>,

    pub global_position: Vector3<f32>,
    pub global_rotation: Quaternion<f32>,
    pub global_scale: Vector3<f32>,
}

impl InspectValue for Transform {
    fn inspect_value(&mut self, ui: &mut egui::Ui) {
        self.inspect(ui);
        self.calculate_rotation();
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Vector3::new(0.0, 0.0, 0.0),
            rotation: Quaternion::one(),
            rotation_euler: Vector3::new(0.0, 0.0, 0.0),
            scale: Vector3::new(1.0, 1.0, 1.0),

            global_position: Vector3::new(0.0, 0.0, 0.0),
            global_rotation: Quaternion::one(),
            global_scale: Vector3::new(1.0, 1.0, 1.0),

            up: Vector3::new(0.0, 1.0, 0.0),
            forward: Vector3::new(0.0, 0.0, -1.0),
            right: Vector3::new(1.0, 0.0, 0.0),
        }
    }
}

impl Transform {
    pub fn calculate_rotation(&mut self) {
        self.rotation = Quaternion::from(Euler {
            x: Deg(0.0),
            y: Deg(self.rotation_euler.y),
            z: Deg(0.0),
        }) * Quaternion::from(Euler {
            x: Deg(self.rotation_euler.x),
            y: Deg(0.0),
            z: Deg(0.0),
        }) * Quaternion::from(Euler {
            x: Deg(0.0),
            y: Deg(0.0),
            z: Deg(self.rotation_euler.z),
        });
    }

    pub fn calculate_up(&self) -> Vector3<f32> {
        self.rotation.rotate_vector(self.up)
    }

    pub fn calculate_forward(&self) -> Vector3<f32> {
        self.rotation.rotate_vector(self.forward)
    }

    pub fn calculate_right(&self) -> Vector3<f32> {
        self.rotation.rotate_vector(self.right)
    }

    pub fn calculate_global_forward(&self) -> Vector3<f32> {
        self.global_rotation.rotate_vector(self.forward)
    }

    pub fn calculate_global_up(&self) -> Vector3<f32> {
        self.global_rotation.rotate_vector(self.up)
    }
    pub fn calculate_global_right(&self) -> Vector3<f32> {
        self.global_rotation.rotate_vector(self.right)
    }
}

#[derive(Clone)]
pub struct ParentGlobal {
    pub position: Vector3<f32>,
    pub rotation: Quaternion<f32>,
    pub scale: Vector3<f32>,
}

impl Default for ParentGlobal {
    fn default() -> Self {
        Self {
            position: Vector3::new(0.0, 0.0, 0.0),
            rotation: Quaternion::one(),
            scale: Vector3::new(1.0, 1.0, 1.0),
        }
    }
}

#[update]
fn propagate_transforms_root(world: &mut World) {
    let root = ParentGlobal::default();
    for node in world.scene.root_node.children.iter_mut() {
        node.propagate_transform(Some(&root));
    }
    for node in world.global_nodes.iter_mut() {
        node.propagate_transform(Some(&root));
    }
}
