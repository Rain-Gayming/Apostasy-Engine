use crate::engine::{
    editor::inspectable::Inspectable,
    nodes::{
        World,
        components::{transform::Transform, velocity::Velocity},
    },
};
use apostasy_macros::{Component, InspectValue, Inspectable, SerializableComponent, update};
use cgmath::{InnerSpace, Quaternion, Vector3};
use serde::{Deserialize, Serialize};

use crate as apostasy;

#[derive(Clone, Debug, PartialEq, InspectValue, Serialize, Deserialize)]
pub enum ColliderShape {
    Cuboid { size: Vector3<f32> },
    Sphere { radius: f32 },
    Capsule { radius: f32, height: f32 },
    Cylinder { radius: f32, height: f32 },
}

impl ColliderShape {
    pub fn half_extents(&self) -> Vector3<f32> {
        match self {
            ColliderShape::Cuboid { size } => *size,
            ColliderShape::Sphere { radius } => Vector3::new(*radius, *radius, *radius),
            ColliderShape::Capsule { radius, height } => {
                Vector3::new(*radius, height * 0.5 + radius, *radius)
            }
            ColliderShape::Cylinder { radius, height } => {
                Vector3::new(*radius, height * 0.5, *radius)
            }
        }
    }
}

impl Inspectable for ColliderShape {
    fn inspect(
        &mut self,
        ui: &mut egui::Ui,
        _editor_storage: &mut crate::engine::editor::EditorStorage,
    ) -> bool {
        egui::ComboBox::from_label("")
            .selected_text(match self {
                ColliderShape::Cuboid { .. } => "Cuboid",
                ColliderShape::Sphere { .. } => "Sphere",
                ColliderShape::Capsule { .. } => "Capsule",
                ColliderShape::Cylinder { .. } => "Cylinder",
            })
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    self,
                    ColliderShape::Cuboid {
                        size: Vector3::new(1.0, 1.0, 1.0),
                    },
                    "Cuboid",
                );
                ui.selectable_value(self, ColliderShape::Sphere { radius: 1.0 }, "Sphere");
                ui.selectable_value(
                    self,
                    ColliderShape::Capsule {
                        radius: 1.0,
                        height: 1.0,
                    },
                    "Capsule",
                );
                ui.selectable_value(
                    self,
                    ColliderShape::Cylinder {
                        radius: 1.0,
                        height: 1.0,
                    },
                    "Cylinder",
                );
            });

        ui.separator();

        match self {
            ColliderShape::Cuboid { size } => {
                ui.label("Size");
                ui.horizontal(|ui| {
                    ui.label("X");
                    ui.add(egui::DragValue::new(&mut size.x).speed(0.01));
                    ui.label("Y");
                    ui.add(egui::DragValue::new(&mut size.y).speed(0.01));
                    ui.label("Z");
                    ui.add(egui::DragValue::new(&mut size.z).speed(0.01));
                });
            }
            ColliderShape::Sphere { radius } => {
                ui.horizontal(|ui| {
                    ui.label("Radius");
                    ui.add(egui::DragValue::new(radius).speed(0.01));
                });
            }
            ColliderShape::Capsule { radius, height } => {
                ui.horizontal(|ui| {
                    ui.label("Radius");
                    ui.add(egui::DragValue::new(radius).speed(0.01));
                });
                ui.horizontal(|ui| {
                    ui.label("Height");
                    ui.add(egui::DragValue::new(height).speed(0.01));
                });
            }
            ColliderShape::Cylinder { radius, height } => {
                ui.horizontal(|ui| {
                    ui.label("Radius");
                    ui.add(egui::DragValue::new(radius).speed(0.01));
                });
                ui.horizontal(|ui| {
                    ui.label("Height");
                    ui.add(egui::DragValue::new(height).speed(0.01));
                });
            }
        }

        false
    }
}

#[derive(
    Component, Clone, Inspectable, InspectValue, SerializableComponent, Serialize, Deserialize,
)]
pub struct Collider {
    pub shape: ColliderShape,
    pub offset: Vector3<f32>,
    pub is_static: bool,
    pub is_area: bool,
}

impl Default for Collider {
    fn default() -> Self {
        Self {
            shape: ColliderShape::Cuboid {
                size: Vector3::new(1.0, 1.0, 1.0),
            },
            offset: Vector3::new(0.0, 0.0, 0.0),
            is_static: false,
            is_area: false,
        }
    }
}

impl Collider {
    /// Creates a dynamic collider
    pub fn new(shape: ColliderShape, offset: Vector3<f32>) -> Self {
        Self {
            shape,
            offset,
            is_static: false,
            is_area: false,
        }
    }

    /// Creates a static collider
    pub fn new_static(shape: ColliderShape, offset: Vector3<f32>) -> Self {
        Self {
            shape,
            offset,
            is_static: true,
            is_area: false,
        }
    }

    /// Returns the world-space center of this collider (offset rotated by entity rotation).
    pub fn world_center(&self, position: Vector3<f32>, rotation: Quaternion<f32>) -> Vector3<f32> {
        position + rotate_vector(rotation, self.offset)
    }

    /// Returns the three local axes of this OBB in world space.
    pub fn world_axes(&self, rotation: Quaternion<f32>) -> [Vector3<f32>; 3] {
        [
            rotate_vector(rotation, Vector3::new(1.0, 0.0, 0.0)),
            rotate_vector(rotation, Vector3::new(0.0, 1.0, 0.0)),
            rotate_vector(rotation, Vector3::new(0.0, 0.0, 1.0)),
        ]
    }

    /// Returns the half-extents (collider_size is already treated as half-extents).
    pub fn half_extents(&self) -> Vector3<f32> {
        self.shape.half_extents()
    }

    pub fn translation_vector_against(
        &self,
        pos_a: Vector3<f32>,
        rotation_a: Quaternion<f32>,
        other: &Collider,
        pos_b: Vector3<f32>,
        rotation_b: Quaternion<f32>,
    ) -> Option<Vector3<f32>> {
        let center_a = self.world_center(pos_a, rotation_a);
        let center_b = other.world_center(pos_b, rotation_b);
        let axes_a = self.world_axes(rotation_a);
        let axes_b = other.world_axes(rotation_b);
        let half_a = self.half_extents();
        let half_b = other.half_extents();

        // The vector from B's center to A's center
        let d = center_a - center_b;

        let mut min_overlap = f32::MAX;
        let mut min_axis = Vector3::new(0.0f32, 0.0, 0.0);

        let face_axes: [Vector3<f32>; 6] = [
            axes_a[0], axes_a[1], axes_a[2], axes_b[0], axes_b[1], axes_b[2],
        ];

        for axis in &face_axes {
            // Skip near-zero axes (shouldn't happen for face normals, but be safe)
            if axis.magnitude2() < 1e-10 {
                continue;
            }
            let axis = axis.normalize();

            let proj_a = project_obb(axis, &axes_a, half_a);
            let proj_b = project_obb(axis, &axes_b, half_b);
            let dist = d.dot(axis).abs();
            let overlap = proj_a + proj_b - dist;

            if overlap <= 0.0 {
                return None; // Separating axis found, no collision
            }
            if overlap < min_overlap {
                min_overlap = overlap;
                // Ensure the MTV points from B toward A
                min_axis = if d.dot(axis) >= 0.0 { axis } else { -axis };
            }
        }

        Some(min_axis * min_overlap)
    }

    pub fn contains_point(
        &self,
        position: Vector3<f32>,
        point: Vector3<f32>,
        rotation: Quaternion<f32>,
    ) -> bool {
        let axes = self.world_axes(rotation);
        let half = self.half_extents();
        let center = self.world_center(position, rotation);
        let local = point - center;

        // Project the point onto each local axis and check against half-extent
        local.dot(axes[0]).abs() <= half.x
            && local.dot(axes[1]).abs() <= half.y
            && local.dot(axes[2]).abs() <= half.z
    }
}

fn project_obb(axis: Vector3<f32>, obb_axes: &[Vector3<f32>; 3], half: Vector3<f32>) -> f32 {
    axis.dot(obb_axes[0]).abs() * half.x
        + axis.dot(obb_axes[1]).abs() * half.y
        + axis.dot(obb_axes[2]).abs() * half.z
}

/// Rotates a vector by a quaternion: q * v * q^-1
fn rotate_vector(q: Quaternion<f32>, v: Vector3<f32>) -> Vector3<f32> {
    let qv = Vector3::new(q.v.x, q.v.y, q.v.z);
    let t = qv.cross(v) * 2.0;
    v + t * q.s + qv.cross(t)
}

/// Contains information about a collision event
#[derive(Debug, Clone, InspectValue, Inspectable)]
pub struct CollisionEvent {
    pub node_a: String,
    pub node_b: String,
    pub translation_vector: Vector3<f32>,
    pub depth: f32,
    pub normal: Vector3<f32>,
}

#[derive(Debug, Clone, Default, Component, InspectValue, Inspectable)]
pub struct CollisionEvents {
    pub events: Vec<CollisionEvent>,
}

impl CollisionEvents {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Data cloned from each eligible node for the read-only detection pass.
#[derive(Clone)]
struct Snapshot {
    name: String,
    position: Vector3<f32>,
    rotation: Quaternion<f32>,
    collider: Collider,
}

fn build_snapshot(world: &World) -> Vec<Snapshot> {
    world
        .get_all_nodes()
        .into_iter()
        .filter_map(|node| {
            let transform = node.get_component::<Transform>()?;
            let position = transform.position;
            let scale = transform.scale;
            let rotation = transform.rotation;
            let mut collider = node.get_component::<Collider>()?.clone();

            // Bake scale into collider_size so collision matches world-space mesh size
            collider.shape = match collider.shape {
                ColliderShape::Cuboid { size } => ColliderShape::Cuboid {
                    size: Vector3::new(size.x * scale.x, size.y * scale.y, size.z * scale.z),
                },
                ColliderShape::Sphere { radius } => ColliderShape::Sphere {
                    radius: radius * scale.x.max(scale.y).max(scale.z),
                },
                ColliderShape::Capsule { radius, height } => ColliderShape::Capsule {
                    radius: radius * scale.x.max(scale.z),
                    height: height * scale.y,
                },
                ColliderShape::Cylinder { radius, height } => ColliderShape::Cylinder {
                    radius: radius * scale.x.max(scale.z),
                    height: height * scale.y,
                },
            };

            Some(Snapshot {
                name: node.name.clone(),
                position,
                rotation,
                collider,
            })
        })
        .collect()
}

/// Detects collisions between all nodes using OBB vs OBB SAT
#[update]
pub fn collision_detection_system(world: &mut World) {
    let snapshot = build_snapshot(world);
    let n = snapshot.len();

    let mut events: Vec<CollisionEvent> = Vec::new();

    for i in 0..n {
        for j in (i + 1)..n {
            let a = &snapshot[i];
            let b = &snapshot[j];

            if let Some(translation_vector) = a.collider.translation_vector_against(
                a.position,
                a.rotation,
                &b.collider,
                b.position,
                b.rotation,
            ) {
                let depth = translation_vector.magnitude();
                let normal = if depth > 1e-10 {
                    translation_vector / depth
                } else {
                    Vector3::new(0.0, 1.0, 0.0)
                };
                events.push(CollisionEvent {
                    node_a: a.name.clone(),
                    node_b: b.name.clone(),
                    translation_vector,
                    depth,
                    normal,
                });
            }
        }
    }

    for event in &events {
        let a = world.get_node_with_name(&event.node_a);
        let b = world.get_node_with_name(&event.node_b);

        if let Some(a) = a
            && let Some(b) = b
            && let Some(a_collider) = a.get_component::<Collider>()
            && let Some(b_collider) = b.get_component::<Collider>()
        {
            let a_static = a_collider.is_static;
            let b_static = b_collider.is_static;

            let normal_a = event.normal;
            let normal_b = -event.normal;

            match (a_static, b_static) {
                // Both dynamic: split the correction evenly
                (false, false) => {
                    let half = event.translation_vector * 0.5;
                    resolve_node(world, &event.node_a, half, normal_a);
                    resolve_node(world, &event.node_b, -half, normal_b);
                }
                // A is static: push B the full amount
                (true, false) => {
                    resolve_node(world, &event.node_b, -event.translation_vector, normal_b);
                }
                // B is static: push A the full amount
                (false, true) => {
                    resolve_node(world, &event.node_a, event.translation_vector, normal_a);
                }
                // Both static: do nothing
                (true, true) => {}
            }
        }
    }

    for global in world.global_nodes.iter_mut() {
        if let Some(ev) = global.get_component_mut::<CollisionEvents>() {
            ev.events = events;
            return;
        }
    }
}

/// Resolves a collision by pushing the node and cancelling inward velocity
fn resolve_node(world: &mut World, name: &str, offset: Vector3<f32>, normal: Vector3<f32>) {
    if let Some(node) = world.get_node_with_name_mut(name) {
        let (transform, velocity, collider) =
            node.get_components_mut::<(&mut Transform, &mut Velocity, &mut Collider)>();

        // Apply positional correction immediately
        transform.position += offset;

        // Cancel the velocity component pointing into the surface
        let v_dot_n = velocity.direction.dot(normal);
        if v_dot_n < 0.0 {
            velocity.direction -= normal * v_dot_n;
        }

        if let ColliderShape::Sphere { radius } = collider.shape {
            velocity.angular_direction += normal.cross(velocity.direction) * (1.0 / radius);
        }
    }
}
