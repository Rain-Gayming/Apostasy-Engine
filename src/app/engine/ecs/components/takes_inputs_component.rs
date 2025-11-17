use crate::app::engine::ecs::{
    component::Component,
    components::{rotation_component::RotationComponent, velocity_component::VelocityComponent},
    resource::{Res, ResMut},
    resources::input_manager::{InputManager, is_keybind_name_triggered},
};
use component_derive::DeriveComponent;

#[derive(DeriveComponent, Clone)]
pub struct TakesInputs {}

pub fn handle_velocity_input(
    velocity_component: &mut VelocityComponent,
    input_manager: &mut ResMut<InputManager>,
) {
    //forwards backwards movement
    if is_keybind_name_triggered(input_manager, "move_forwards".to_string()) {
        velocity_component.velocity.z = -1.0;
    } else if is_keybind_name_triggered(input_manager, "move_backwards".to_string()) {
        velocity_component.velocity.z = 1.0;
    } else {
        velocity_component.velocity.z = 0.0;
    }

    //  left right movement
    if is_keybind_name_triggered(input_manager, "move_left".to_string()) {
        velocity_component.velocity.x = -1.0;
    } else if is_keybind_name_triggered(input_manager, "move_right".to_string()) {
        velocity_component.velocity.x = 1.0;
    } else {
        velocity_component.velocity.x = 0.0;
    }

    //  up down movement
    if is_keybind_name_triggered(input_manager, "move_jump".to_string()) {
        velocity_component.velocity.y = -1.0;
    } else if is_keybind_name_triggered(input_manager, "move_crouch".to_string()) {
        velocity_component.velocity.y = 1.0;
    } else {
        velocity_component.velocity.y = 0.0;
    }
}
pub fn handle_rotation_input(
    rotation_component: &mut RotationComponent,
    input_manager: Res<InputManager>,
) {
    rotation_component.yaw += input_manager.mouse_delta[0] as f32 / 10.0;
    rotation_component.pitch += -input_manager.mouse_delta[1] as f32 / 10.0;
    rotation_component.pitch = rotation_component.pitch.clamp(-80.0, 80.0);
}
