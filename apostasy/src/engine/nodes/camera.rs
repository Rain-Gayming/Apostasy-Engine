use crate::engine::editor::inspectable::Inspectable;
use crate::{self as apostasy};
use apostasy_macros::{Component, Inspectable, SerializableComponent};
use cgmath::{Deg, Matrix4, PerspectiveFov};
use serde::{Deserialize, Serialize};

#[derive(Component, Clone, Inspectable, SerializableComponent, Serialize, Deserialize)]
pub struct Camera {
    pub fov_y: f32,
    pub near: f32,
    pub far: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            fov_y: 90.0,
            far: 10000.0,
            near: 0.001,
        }
    }
}
pub fn get_perspective_projection(camera: &Camera, aspect: f32) -> Matrix4<f32> {
    let mut proj: Matrix4<f32> = PerspectiveFov::to_perspective(&PerspectiveFov {
        fovy: Deg(camera.fov_y).into(),
        aspect,
        near: camera.near,
        far: camera.far,
    })
    .into();

    proj[1][1] *= -1.0;

    proj
}
