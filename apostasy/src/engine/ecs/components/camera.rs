use crate as apostasy;
use apostasy_macros::Component;
use cgmath::{Deg, Matrix4, PerspectiveFov, Zero};

#[derive(Component)]
pub struct Camera {
    pub fov_y: f32,
    pub near: f32,
    pub far: f32,
    pub projection: Matrix4<f32>,
}
impl Default for Camera {
    fn default() -> Self {
        Self {
            fov_y: 90.0,
            far: 10000.0,
            near: 0.001,
            projection: Matrix4::<f32>::zero(),
        }
    }
}

pub fn get_perspective_projection(camera: &Camera, aspect: f32) -> Matrix4<f32> {
    PerspectiveFov::to_perspective(&PerspectiveFov {
        fovy: Deg(camera.fov_y).into(),
        aspect,
        near: camera.near,
        far: camera.far,
    })
    .into()
}
