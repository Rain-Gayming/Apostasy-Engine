use std::any::TypeId;

use cgmath::Matrix4;

use crate::app::engine::ecs::component::Component;

pub struct CameraComponent {
    pub far: f32,
    pub near: f32,
    pub fovy: f32,
    pub projection_matrix: Matrix4<f32>,
}

impl Component for CameraComponent {
    fn type_id_dyn(&self) -> TypeId {
        TypeId::of::<CameraComponent>()
    }
}
