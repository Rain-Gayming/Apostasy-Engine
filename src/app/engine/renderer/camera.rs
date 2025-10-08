use nalgebra::Vector3;

pub struct Camera {
    pub position: Vector3<f32>,
    pub velocoity: Vector3<f32>,
    pub pitch: f32,
    pub yaw: f32,
}

impl Camera {
    pub fn new(position: Vector3<f32>, pitch: f32, yaw: f32) -> Self {
        Self {
            position,
            velocoity: Vector3::new(0.0, 0.0, 0.0),
            pitch,
            yaw,
        }
    }
}

pub fn get_view_matrix(camera: &Camera) -> nalgebra::Matrix4<f32> {
    let rotation = nalgebra::Rotation3::from_euler_angles(
        camera.pitch.to_radians(),
        camera.yaw.to_radians(),
        0.0,
    );
    let translation = nalgebra::Translation3::from(-camera.position);
    (rotation.to_homogeneous() * translation.to_homogeneous())
        .try_inverse()
        .unwrap()
}

pub fn get_rotation_matrix(camera: &Camera) -> nalgebra::Matrix4<f32> {
    let rotation = nalgebra::Rotation3::from_euler_angles(
        camera.pitch.to_radians(),
        camera.yaw.to_radians(),
        0.0,
    );
    rotation.to_homogeneous()
}
