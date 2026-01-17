use apostasy::engine::{
    ecs::{
        World,
        components::{camera::Camera, transform::Transform},
    },
    rendering::start_renderer,
};
use cgmath::{Deg, Quaternion, Rotation3, Vector3};

fn main() {
    let world = World::new();

    // Rotate camera to look down at cube from an angle
    let rotation = Quaternion::from_axis_angle(Vector3::new(1.0, 0.0, 0.0), Deg(35.0));

    world.spawn().insert(Camera::default()).insert(Transform {
        position: Vector3::new(0.0, -2.0, 2.0),
        rotation,
    });

    start_renderer(world).unwrap();
}
