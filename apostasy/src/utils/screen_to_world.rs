use cgmath::{Rotation, Vector3};

use crate::engine::nodes::{
    components::{
        camera::{Camera, get_perspective_projection},
        raycast::ray_from_mouse,
        transform::Transform,
    },
    world::World,
};

pub fn screen_to_world_plane(
    screen_pos: egui::Pos2,
    viewport_rect: egui::Rect,
    world: &World,
    _context: &egui::Context,
) -> Vector3<f32> {
    use cgmath::InnerSpace;

    let camera_node = match world.get_global_node_with_component::<Camera>() {
        Some(n) => n,
        None => return Vector3::new(0.0, 0.0, 0.0),
    };

    let cam = camera_node.get_component::<Camera>().unwrap();
    let transform = camera_node.get_component::<Transform>().unwrap();

    let viewport_width = viewport_rect.width();
    let viewport_height = viewport_rect.height();
    let aspect = viewport_width / viewport_height;

    // Match ray_from_mouse exactly — mouse relative to viewport top-left
    let mouse_x = screen_pos.x - viewport_rect.min.x;
    let mouse_y = screen_pos.y - viewport_rect.min.y;

    let (origin, direction) = ray_from_mouse(
        mouse_x,
        mouse_y,
        viewport_width,
        viewport_height,
        get_perspective_projection(cam, aspect),
        transform.global_position,
        transform.global_rotation,
    );

    // Intersect with Y=0 world plane
    let t = if direction.y.abs() > 1e-6 {
        -origin.y / direction.y
    } else {
        let forward = transform
            .global_rotation
            .rotate_vector(Vector3::new(0.0, 0.0, -1.0));
        let cam_forward_flat = Vector3::new(forward.x, 0.0, forward.z).normalize();
        return origin + cam_forward_flat * 10.0;
    };

    let result = origin + direction * t;

    result
}
