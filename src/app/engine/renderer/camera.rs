use std::sync::{Arc, Mutex};

use nalgebra::{Matrix4, Rotation3, Vector3, Vector4};

use crate::app::engine::input_manager::{is_keybind_name_triggered, InputManager};

#[derive(Clone, Copy)]
pub struct Camera {
    pub position: Vector3<f32>,
    pub velocity: Vector3<f32>,
    pub pitch: f32,
    pub yaw: f32,
    pub far: f32,
    pub near: f32,
    pub fovy: f32,
    pub projection_matrix: Matrix4<f32>,
}

impl Camera {
    pub fn new(position: Vector3<f32>) -> Self {
        Self {
            position,
            velocity: Vector3::new(0.0, 0.0, 0.0),
            pitch: 0.0,
            yaw: 0.0,
            far: 10000.0,
            near: 0.001,
            fovy: 90.0,
            projection_matrix: Matrix4::<f32>::zeros(),
        }
    }
}

pub fn get_perspective_projection(camera: Arc<Mutex<Camera>>, aspect: f32) -> Matrix4<f32> {
    let mut camera = camera.lock().unwrap();
    assert!((aspect - f32::EPSILON) > 0.0);
    let tan_half_fovy: f32 = (camera.fovy / 2.0).tan();
    camera.projection_matrix = Matrix4::new(
        0.0, 0.0, 0.0, 0.0, //
        0.0, 0.0, 0.0, 0.0, //
        0.0, 0.0, 0.0, 0.0, //
        0.0, 0.0, 0.0, 0.0, //
    );

    camera.projection_matrix[0] = 1.0 / (aspect * tan_half_fovy);
    camera.projection_matrix[5] = 1.0 / (tan_half_fovy);
    camera.projection_matrix[10] = camera.far / (camera.far - camera.near);
    camera.projection_matrix[11] = 1.0;
    camera.projection_matrix[14] = -(camera.far * camera.near) / (camera.far - camera.near);

    camera.projection_matrix
}

pub fn get_view_matrix(camera: Arc<Mutex<Camera>>) -> nalgebra::Matrix4<f32> {
    let camera = camera.lock().unwrap();

    let yaw = Rotation3::from_axis_angle(&Vector3::y_axis(), camera.yaw.to_radians());
    let pitch = Rotation3::from_axis_angle(&Vector3::x_axis(), camera.pitch.to_radians());
    let rotation = yaw * pitch; // Yaw first, then pitch

    let translation = nalgebra::Translation3::from(-camera.position);
    drop(camera);
    println!("{}", (rotation * translation).inverse());
    (rotation * translation).inverse().into()
}

pub fn get_rotation_matrix(camera: Arc<Mutex<Camera>>) -> nalgebra::Matrix4<f32> {
    let camera = camera.lock().unwrap();
    let pitch_rotation = Rotation3::from_axis_angle(&Vector3::x_axis(), camera.pitch);
    let yaw_rotation = Rotation3::from_axis_angle(&-Vector3::y_axis(), camera.yaw);
    drop(camera);

    Matrix4::from(yaw_rotation) * Matrix4::from(pitch_rotation)
}

pub fn update_camera(camera: Arc<Mutex<Camera>>) {
    let camera_rotation = get_rotation_matrix(camera.clone());

    let mut camera = camera.lock().unwrap();

    let movement_matrix = camera_rotation
        * Vector4::new(
            camera.velocity.x * 0.5,
            camera.velocity.y * 0.5,
            camera.velocity.z * 0.5,
            0.0,
        );
    let movement_vector = Vector3::new(movement_matrix.x, movement_matrix.y, movement_matrix.z);

    camera.position += movement_vector;
    // println!("{}", camera.position);
    // println!("{}", camera_rotation);
    drop(camera);
}

pub fn handle_camera_input(input_manager: &InputManager, camera: &mut Arc<Mutex<Camera>>) {
    let mut camera = camera.lock().unwrap();
    //forwards backwards movement
    if is_keybind_name_triggered(input_manager, "move_forwards".to_string()) {
        camera.velocity.z = -1.0;
    } else if is_keybind_name_triggered(input_manager, "move_backwards".to_string()) {
        camera.velocity.z = 1.0;
    } else {
        camera.velocity.z = 0.0;
    }

    //  left right movement
    if is_keybind_name_triggered(input_manager, "move_left".to_string()) {
        camera.velocity.x = -1.0;
    } else if is_keybind_name_triggered(input_manager, "move_right".to_string()) {
        camera.velocity.x = 1.0;
    } else {
        camera.velocity.x = 0.0;
    }

    //  up down movement
    if is_keybind_name_triggered(input_manager, "move_jump".to_string()) {
        camera.velocity.y = -1.0;
    } else if is_keybind_name_triggered(input_manager, "move_crouch".to_string()) {
        camera.velocity.y = 1.0;
    } else {
        camera.velocity.y = 0.0;
    }

    camera.yaw += input_manager.mouse_delta[0] as f32 / 200.0;
    camera.pitch += input_manager.mouse_delta[1] as f32 / 200.0;
    drop(camera);
}
