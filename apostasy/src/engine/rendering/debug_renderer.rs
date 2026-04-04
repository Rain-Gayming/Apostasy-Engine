use ash::vk;
use cgmath::{Quaternion, Vector3};

use crate::engine::physics::collider::{Collider, ColliderShape, CollisionEvents};

#[repr(C)]
#[derive(Clone, Copy)]
pub struct DebugLineVertex {
    pub position: [f32; 3],
    pub color: [f32; 4],
}

pub struct DebugLine {
    pub start: Vector3<f32>,
    pub end: Vector3<f32>,
    pub color: [f32; 4],
}

pub struct DebugRendererSettings {
    pub line_thickness: f32,
}

pub struct DebugRenderer {
    pub lines: Vec<DebugLine>,
    pub vertex_buffer: vk::Buffer,
    pub vertex_buffer_memory: vk::DeviceMemory,
    pub vertex_capacity: usize,
    pub pipeline: vk::Pipeline,
    pub pipeline_layout: vk::PipelineLayout,
    pub enabled: bool,
}

impl DebugRenderer {
    pub fn clear(&mut self) {
        self.lines.clear();
    }

    pub fn draw_collider(
        &mut self,
        collider: &Collider,
        position: Vector3<f32>,
        rotation: Quaternion<f32>,
        scale: Vector3<f32>,
        collision_events: &CollisionEvents,
        node_name: &str,
    ) {
        let color = if collider.is_area {
            [0.2, 0.4, 1.0, 0.8]
        } else if collision_events
            .events
            .iter()
            .any(|e| e.node_a == node_name || e.node_b == node_name)
        {
            [1.0, 0.2, 0.2, 0.8]
        } else if collider.is_static {
            [1.0, 0.85, 0.1, 0.8]
        } else {
            [0.2, 1.0, 0.3, 0.8]
        };

        let center = collider.world_center(position, rotation);
        let axes = collider.world_axes(rotation);

        match &collider.shape {
            ColliderShape::Cuboid { size } => {
                let scaled_half =
                    Vector3::new(size.x * scale.x, size.y * scale.y, size.z * scale.z);
                self.draw_obb(center, axes, scaled_half, color);
            }
            ColliderShape::Sphere { radius } => {
                let scaled_radius = radius * scale.x.max(scale.y).max(scale.z);
                self.draw_circle(center, axes[0], axes[1], scaled_radius, 24, color);
                self.draw_circle(center, axes[1], axes[2], scaled_radius, 24, color);
                self.draw_circle(center, axes[0], axes[2], scaled_radius, 24, color);
            }
            ColliderShape::Capsule { radius, height } => {
                let scaled_radius = radius * scale.x.max(scale.z);
                let scaled_half_h = height * scale.y * 0.5;

                let top = center + axes[1] * scaled_half_h;
                let bot = center - axes[1] * scaled_half_h;

                self.draw_circle(top, axes[0], axes[2], scaled_radius, 24, color);
                self.draw_circle(bot, axes[0], axes[2], scaled_radius, 24, color);

                for dir in [axes[0], -axes[0], axes[2], -axes[2]] {
                    self.add_line(top + dir * scaled_radius, bot + dir * scaled_radius, color);
                }

                // Hemispheres at caps
                self.draw_half_circle(top, axes[1], axes[0], scaled_radius, 24, color);
                self.draw_half_circle(top, axes[1], axes[2], scaled_radius, 24, color);
                self.draw_half_circle(bot, -axes[1], axes[0], scaled_radius, 24, color);
                self.draw_half_circle(bot, -axes[1], axes[2], scaled_radius, 24, color);
            }
            ColliderShape::Cylinder { radius, height } => {
                let scaled_radius = radius * scale.x.max(scale.z);
                let scaled_half_h = height * scale.y * 0.5;

                let top = center + axes[1] * scaled_half_h;
                let bot = center - axes[1] * scaled_half_h;

                self.draw_circle(top, axes[0], axes[2], scaled_radius, 24, color);
                self.draw_circle(bot, axes[0], axes[2], scaled_radius, 24, color);

                for dir in [axes[0], -axes[0], axes[2], -axes[2]] {
                    self.add_line(top + dir * scaled_radius, bot + dir * scaled_radius, color);
                }
            }
        }
    }

    fn draw_half_circle(
        &mut self,
        center: Vector3<f32>,
        up: Vector3<f32>,
        side: Vector3<f32>,
        radius: f32,
        segments: usize,
        color: [f32; 4],
    ) {
        for i in 0..segments {
            let t0 = (i as f32 / segments as f32) * std::f32::consts::PI;
            let t1 = ((i + 1) as f32 / segments as f32) * std::f32::consts::PI;
            let p0 = center + up * (t0.cos() * radius) + side * (t0.sin() * radius);
            let p1 = center + up * (t1.cos() * radius) + side * (t1.sin() * radius);
            self.add_line(p0, p1, color);
        }
    }
    fn draw_obb(
        &mut self,
        center: Vector3<f32>,
        axes: [Vector3<f32>; 3],
        half: Vector3<f32>,
        color: [f32; 4],
    ) {
        let corners: Vec<Vector3<f32>> = (0..8)
            .map(|i| {
                center
                    + axes[0] * if i & 1 != 0 { half.x } else { -half.x }
                    + axes[1] * if i & 2 != 0 { half.y } else { -half.y }
                    + axes[2] * if i & 4 != 0 { half.z } else { -half.z }
            })
            .collect();

        let edges = [
            (0, 1),
            (2, 3),
            (4, 5),
            (6, 7), // X edges
            (0, 2),
            (1, 3),
            (4, 6),
            (5, 7), // Y edges
            (0, 4),
            (1, 5),
            (2, 6),
            (3, 7), // Z edges
        ];
        for (a, b) in edges {
            self.add_line(corners[a], corners[b], color);
        }
    }

    fn draw_circle(
        &mut self,
        center: Vector3<f32>,
        axis_a: Vector3<f32>,
        axis_b: Vector3<f32>,
        radius: f32,
        segments: usize,
        color: [f32; 4],
    ) {
        for i in 0..segments {
            let t0 = (i as f32 / segments as f32) * std::f32::consts::TAU;
            let t1 = ((i + 1) as f32 / segments as f32) * std::f32::consts::TAU;
            let p0 = center + axis_a * (t0.cos() * radius) + axis_b * (t0.sin() * radius);
            let p1 = center + axis_a * (t1.cos() * radius) + axis_b * (t1.sin() * radius);
            self.add_line(p0, p1, color);
        }
    }

    fn add_line(&mut self, start: Vector3<f32>, end: Vector3<f32>, color: [f32; 4]) {
        self.lines.push(DebugLine { start, end, color });
    }
}
