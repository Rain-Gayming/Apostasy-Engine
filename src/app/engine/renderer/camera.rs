use std::sync::{Arc, Mutex};

use cgmath::{
    Deg, Euler, Matrix4, One, PerspectiveFov, Quaternion, Rotation, Rotation3, Vector3, Vector4,
    Zero,
};

use crate::app::engine::input_manager::{is_keybind_name_triggered, InputManager};

#[derive(Clone, Copy)]
pub struct Camera {
    pub position: Vector3<f32>,
    pub rotation: Quaternion<f32>,
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
            rotation: Quaternion::one(),
            velocity: Vector3::new(0.0, 0.0, 0.0),
            pitch: 0.0,
            yaw: 0.0,
            far: 10000.0,
            near: 0.001,
            fovy: 90.0,
            projection_matrix: Matrix4::<f32>::zero(),
        }
    }
}

pub fn get_perspective_projection(camera: Arc<Mutex<Camera>>, aspect: f32) -> Matrix4<f32> {
    let mut camera = camera.lock().unwrap();

    let perspective = PerspectiveFov::to_perspective(&PerspectiveFov {
        fovy: cgmath::Deg(camera.fovy).into(),
        aspect,
        near: camera.near,
        far: camera.far,
    })
    .into();

    camera.projection_matrix = perspective;

    drop(camera);

    perspective
}

pub fn get_view_matrix(camera: Arc<Mutex<Camera>>) -> Matrix4<f32> {
    let camera = camera.lock().unwrap();
    let view = Matrix4::from(Quaternion::from_sv(
        camera.rotation.s,
        Vector3::new(
            camera.rotation.v.x,
            camera.rotation.v.y,
            camera.rotation.v.z,
        ),
    )) * Matrix4::from_translation(Vector3::new(
        -camera.position.x,
        -camera.position.y,
        -camera.position.z,
    ));
    drop(camera);
    view
}

pub fn rotate_camera(camera: &mut Camera) {
    camera.rotation = Quaternion::from(Euler {
        x: Deg(camera.pitch),
        y: Deg(camera.yaw),
        z: Deg(0.0),
    });
}

pub fn update_camera_position(camera: Arc<Mutex<Camera>>) -> bool {
    let mut camera = camera.lock().unwrap();

    let header = [camera.pitch, camera.yaw];

    let movement_matrix = Quaternion::from_angle_y(Deg(-header[1]))
        * Vector3::new(camera.velocity.x, camera.velocity.y, camera.velocity.z);
    let movement_vector = Vector3::new(movement_matrix.x, movement_matrix.y, movement_matrix.z);

    camera.position += movement_vector / 50.0;
    drop(camera);

    movement_vector != Vector3::zero()
}
pub fn handle_camera_input(input_manager: &mut InputManager, camera: &mut Arc<Mutex<Camera>>) {
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

    camera.yaw += input_manager.mouse_delta[0] as f32 / 10.0;
    camera.pitch += -input_manager.mouse_delta[1] as f32 / 10.0;
    camera.pitch = camera.pitch.clamp(-80.0, 80.0);
    rotate_camera(&mut camera);
    drop(camera);
}
