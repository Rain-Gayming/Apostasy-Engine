use std::mem::transmute;

use cgmath::{Matrix, Matrix4, SquareMatrix, Vector3, Zero};

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
    pub world_position: Vector3<i32>,
}

impl Default for PushConstants {
    fn default() -> Self {
        Self {
            view_matrix: Matrix4::identity(),
            projection_matrix: Matrix4::identity(),
            model_matrix: Matrix4::identity(),
            atlas_tiles: 1,
            world_position: Vector3::zero(),
        }
    }
}

impl PushConstants {
    pub fn return_renderable(&self) -> Vec<u8> {
        unsafe {
            let mut data = Vec::with_capacity(156);
            let proj_view: [u8; 64] = transmute(self.projection_matrix * self.view_matrix);
            let model: [u8; 64] = transmute(self.model_matrix);
            let atlas: [u8; 4] = transmute(self.atlas_tiles);
            let pad: [u8; 12] = [0u8; 12]; // 12 bytes padding to align ivec3
            let position: [u8; 12] = transmute(self.world_position);
            data.extend_from_slice(&proj_view); // offset 0
            data.extend_from_slice(&model); // offset 64
            data.extend_from_slice(&atlas); // offset 128
            data.extend_from_slice(&pad); // offset 132
            data.extend_from_slice(&position); // offset 144
            data // 156 bytes
        }
    }

    pub fn set_position(&mut self, position: Vector3<i32>) {
        self.world_position = position;
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
