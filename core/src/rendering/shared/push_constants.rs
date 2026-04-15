use std::mem::transmute;

use anyhow::Result;
use cgmath::{Matrix4, SquareMatrix};

use crate::{
    objects::{Object, components::transform::Transform},
    rendering::components::camera::{Camera, get_perspective_projection, get_view_matrix},
};

#[derive(Clone)]
pub struct PushConstants {
    pub view_matrix: Matrix4<f32>,
    pub projection_matrix: Matrix4<f32>,
    pub model_matrix: Matrix4<f32>,
}

impl Default for PushConstants {
    fn default() -> Self {
        Self {
            view_matrix: Matrix4::identity(),
            projection_matrix: Matrix4::identity(),
            model_matrix: Matrix4::identity(),
        }
    }
}

impl PushConstants {
    pub fn return_renderable(&self) -> [u8; 128] {
        unsafe {
            let mut push_constants = [0u8; 128];

            let mvp: [u8; 64] =
                transmute(self.projection_matrix * self.view_matrix * self.model_matrix);
            let model: [u8; 64] = transmute(self.model_matrix);

            push_constants[0..64].copy_from_slice(&mvp);
            push_constants[64..128].copy_from_slice(&model);

            push_constants
        }
    }

    pub fn set_camera_constants(&mut self, camera: Object, aspect: f32) -> Result<()> {
        let transform = camera.get_component::<Transform>().unwrap();
        let cam = camera.get_component::<Camera>().unwrap();

        let view = get_view_matrix(transform);
        let proj = get_perspective_projection(cam, aspect);
        let model_matrix = Matrix4::<f32>::identity();

        self.view_matrix = view;
        self.projection_matrix = proj;
        self.model_matrix = model_matrix;

        Ok(())
    }
}
