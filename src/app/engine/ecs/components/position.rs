use crate::app::engine::ecs::{
    Component,
    components::{rotation::RotationComponent, velocity::VelocityComponent},
};
use cgmath::{Deg, Quaternion, Rotation3, Vector3, Zero};
use component_derive::DeriveComponent;

#[derive(Clone, DeriveComponent)]
pub struct PositionComponent {
    pub position: Vector3<f32>,
}

impl Default for PositionComponent {
    fn default() -> Self {
        PositionComponent {
            position: Vector3::zero(),
        }
    }
}

pub fn move_position_with_velocity(
    position_component: &mut PositionComponent,
    rotation_component: &RotationComponent,
    velocity_component: &VelocityComponent,
) {
    let header = [rotation_component.pitch, rotation_component.yaw];

    let movement_matrix = Quaternion::from_angle_y(Deg(-header[1]))
        * Vector3::new(
            velocity_component.velocity.x,
            velocity_component.velocity.y,
            velocity_component.velocity.z,
        );
    let movement_vector = Vector3::new(movement_matrix.x, movement_matrix.y, movement_matrix.z);

    position_component.position += movement_vector / 50.0;
}
