use cgmath::{Matrix4, PerspectiveFov, Quaternion, Vector3, Zero};
use component_derive::DeriveComponent;

use crate::app::engine::ecs::{
    component::Component,
    components::{position::PositionComponent, rotation::RotationComponent},
};

#[derive(Clone, DeriveComponent)]
pub struct CameraComponent {
    pub far: f32,
    pub near: f32,
    pub fovy: f32,
    pub projection_matrix: Matrix4<f32>,
}

impl Default for CameraComponent {
    fn default() -> Self {
        Self {
            far: 10000.0,
            near: 0.001,
            fovy: 90.0,
            projection_matrix: Matrix4::<f32>::zero(),
        }
    }
}

pub fn get_perspective_projection(camera: &mut CameraComponent) -> Matrix4<f32> {
    let perspective = PerspectiveFov::to_perspective(&PerspectiveFov {
        fovy: cgmath::Deg(camera.fovy).into(),
        // aspect,
        aspect: 1.0,
        near: camera.near,
        far: camera.far,
    })
    .into();

    camera.projection_matrix = perspective;

    perspective
}

pub fn get_view_matrix(
    rotation_component: &RotationComponent,
    position_component: &PositionComponent,
) -> Matrix4<f32> {
    Matrix4::from(Quaternion::from_sv(
        rotation_component.rotation.s,
        Vector3::new(
            rotation_component.rotation.v.x,
            rotation_component.rotation.v.y,
            rotation_component.rotation.v.z,
        ),
    )) * Matrix4::from_translation(Vector3::new(
        -position_component.position.x,
        -position_component.position.y,
        -position_component.position.z,
    ))
}
