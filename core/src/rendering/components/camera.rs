use apostasy_macros::Component;
use cgmath::{Deg, Matrix4, PerspectiveFov};

use crate::objects::components::transform::Transform;

#[derive(Component, Clone)]
pub struct Camera {
    pub fov_y: f32,
    pub near: f32,
    pub far: f32,
    pub is_main: bool,
}

impl Default for Camera {
    fn default() -> Self {
        Camera {
            fov_y: 90.0,
            near: 0.001,
            far: 10000.0,
            is_main: false,
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

pub fn get_view_matrix(transform: &Transform) -> Matrix4<f32> {
    let translation = Matrix4::from_translation(-transform.global_position);
    let rotation = Matrix4::from(transform.global_rotation.conjugate());
    rotation * translation
}
