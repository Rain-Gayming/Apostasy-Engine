use crate as apostasy;
use apostasy_macros::Component;
use cgmath::{Deg, Matrix4, PerspectiveFov};

#[derive(Component)]
pub struct Camera {
    pub fov_y: f32,
    pub near: f32,
    pub far: f32,
    pub projection: Matrix4<f32>,
}


pub fn get_perspective_projection(camera: &Camera, aspect: f32) -> Matrix4<f32> {
    let perspective = PerspectiveFov::to_perspective(&PerspectiveFov{
        fovy: Deg(camera.fov_y).into(),
        aspect,
        near: camera.near,
        far: camera.far,
    }).into();
}) 
