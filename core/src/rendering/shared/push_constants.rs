use std::mem::transmute;

use cgmath::{Matrix, Matrix4, SquareMatrix};

use crate::{
    objects::{Object, components::transform::Transform},
    rendering::components::camera::{Camera, get_perspective_projection, get_view_matrix},
};

#[derive(Clone, Debug)]
pub struct PushConstants {
    pub view_matrix: Matrix4<f32>,
    pub projection_matrix: Matrix4<f32>,
    pub model_matrix: Matrix4<f32>,
    pub atlas_tiles: u32, // how many tiles per row in the atlas
}

impl Default for PushConstants {
    fn default() -> Self {
        Self {
            view_matrix: Matrix4::identity(),
            projection_matrix: Matrix4::identity(),
            model_matrix: Matrix4::identity(),
            atlas_tiles: 1,
        }
    }
}

impl PushConstants {
    #[allow(unnecessary_transmutes)]
    pub fn return_renderable(&self) -> Vec<u8> {
        unsafe {
            let mut data = Vec::with_capacity(196);
            let proj_view: [u8; 64] = transmute(self.projection_matrix * self.view_matrix);
            let model: [u8; 64] = transmute(self.model_matrix);
            let atlas: [u8; 4] = transmute(self.atlas_tiles);
            data.extend_from_slice(&proj_view);
            data.extend_from_slice(&model);
            data.extend_from_slice(&atlas);
            data.extend_from_slice(&[0u8; 12]);
            data
        }
    }

    pub fn set_camera_constants(&mut self, camera: Object, aspect: f32) {
        let transform = camera.get_component::<Transform>().unwrap();
        let cam = camera.get_component::<Camera>().unwrap();
        self.view_matrix = get_view_matrix(transform);
        self.projection_matrix = get_perspective_projection(cam, aspect);
        self.model_matrix = Matrix4::identity();
    }

    pub fn set_atlas_tiles(&mut self, tiles: u32) {
        self.atlas_tiles = tiles;
    }
}
