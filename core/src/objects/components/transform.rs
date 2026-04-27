use anyhow::Result;
use apostasy_macros::{Component, update};
use cgmath::{Deg, Euler, Quaternion, Rotation, Vector3};

use crate::objects::world::World;

const UP: Vector3<f32> = Vector3::new(0.0, 1.0, 0.0);
const RIGHT: Vector3<f32> = Vector3::new(1.0, 0.0, 0.0);
const FORWARD: Vector3<f32> = Vector3::new(0.0, 0.0, -1.0);

#[derive(Component, Clone, Debug)]
pub struct Transform {
    pub local_position: Vector3<f32>,
    pub local_euler_angles: Vector3<f32>,
    pub local_rotation: Quaternion<f32>,
    pub local_scale: Vector3<f32>,
    pub global_position: Vector3<f32>,
    pub global_rotation: Quaternion<f32>,
    pub global_euler_angles: Vector3<f32>,
    pub global_scale: Vector3<f32>,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            local_position: Vector3::new(0.0, 0.0, 0.0),
            local_euler_angles: Vector3::new(0.0, 0.0, 0.0),
            local_rotation: Quaternion::new(1.0, 0.0, 0.0, 0.0),
            local_scale: Vector3::new(1.0, 1.0, 1.0),
            global_position: Vector3::new(0.0, 0.0, 0.0),
            global_rotation: Quaternion::new(1.0, 0.0, 0.0, 0.0),
            global_euler_angles: Vector3::new(0.0, 0.0, 0.0),
            global_scale: Vector3::new(1.0, 1.0, 1.0),
        }
    }
}

impl Transform {
    pub fn deserialize(&mut self, value: &serde_yaml::Value) -> anyhow::Result<()> {
        Ok(())
    }
    pub fn calculate_up(&self) -> Vector3<f32> {
        self.local_rotation.rotate_vector(UP)
    }

    pub fn calculate_forward(&self) -> Vector3<f32> {
        self.local_rotation.rotate_vector(FORWARD)
    }

    pub fn calculate_right(&self) -> Vector3<f32> {
        self.local_rotation.rotate_vector(RIGHT)
    }

    pub fn calculate_global_forward(&self) -> Vector3<f32> {
        self.global_rotation.rotate_vector(FORWARD)
    }

    pub fn calculate_global_up(&self) -> Vector3<f32> {
        self.global_rotation.rotate_vector(UP)
    }
    pub fn calculate_global_right(&self) -> Vector3<f32> {
        self.global_rotation.rotate_vector(RIGHT)
    }
}

#[update]
pub fn transform_update(world: &mut World) -> Result<()> {
    let transforms = world.get_objects_with_component_mut::<Transform>();

    for transform in transforms {
        let transform = transform.get_component_mut::<Transform>()?;

        transform.local_rotation = Quaternion::from(Euler {
            x: Deg(0.0),
            y: Deg(transform.local_euler_angles.y),
            z: Deg(0.0),
        }) * Quaternion::from(Euler {
            x: Deg(transform.local_euler_angles.x),
            y: Deg(0.0),
            z: Deg(0.0),
        }) * Quaternion::from(Euler {
            x: Deg(0.0),
            y: Deg(0.0),
            z: Deg(transform.local_euler_angles.z),
        });

        transform.global_rotation = transform.local_rotation;

        transform.global_position = transform.local_position;
        transform.global_scale = transform.local_scale;
        transform.global_euler_angles = transform.local_euler_angles;
    }

    Ok(())
}
