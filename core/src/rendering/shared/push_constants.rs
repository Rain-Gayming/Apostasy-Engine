use std::mem::transmute;

use cgmath::Matrix4;

#[derive(Clone)]
pub struct PushConstants {
    pub view_matrix: Matrix4<f32>,
    pub projection_matrix: Matrix4<f32>,
    pub model_matrix: Matrix4<f32>,
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
}
