use cgmath::{InnerSpace, Matrix4, Quaternion, Rotation, SquareMatrix, Vector3, Vector4};
use serde::{Deserialize, Serialize};

use crate::engine::nodes::{
    Node,
    components::{collider::Collider, transform::Transform},
};
use crate::{self as apostasy};
use apostasy_macros::{Component, Inspectable, SerializableComponent};

#[derive(Debug, Clone)]
pub struct RayHit {
    pub node_name: String,
    pub point: Vector3<f32>,
    pub normal: Vector3<f32>,
    pub distance: f32,
    pub is_static: bool,
    pub is_area: bool,
}

#[derive(Component, Clone, Inspectable, SerializableComponent, Serialize, Deserialize)]
pub struct Raycast {
    pub direction: Vector3<f32>,
    pub max_distance: f32,
}

impl Default for Raycast {
    fn default() -> Self {
        Self {
            direction: Vector3::new(0.0, 0.0, -1.0),
            max_distance: 1.0,
        }
    }
}

impl Raycast {
    pub fn new(direction: Vector3<f32>, max_distance: f32) -> Self {
        Self {
            direction: direction.normalize(),
            max_distance,
        }
    }

    pub fn cast(&self, transform: &Transform, nodes: &[&Node], ignore: &str) -> Option<RayHit> {
        let world_dir = transform.rotation.rotate_vector(self.direction).normalize();
        self.hits(transform.position, world_dir, nodes, |name| name != ignore)
            .into_iter()
            .next()
    }

    pub fn cast_all(&self, transform: &Transform, nodes: &[&Node], ignore: &str) -> Vec<RayHit> {
        let world_dir = transform.rotation.rotate_vector(self.direction).normalize();
        self.hits(transform.position, world_dir, nodes, |name| name != ignore)
    }

    pub fn cast_ignore_many(
        &self,
        transform: &Transform,
        nodes: &[&Node],
        ignore: &[&str],
    ) -> Option<RayHit> {
        let world_dir = transform.rotation.rotate_vector(self.direction).normalize();
        self.hits(transform.position, world_dir, nodes, |name| {
            !ignore.contains(&name)
        })
        .into_iter()
        .next()
    }

    pub fn cast_from(
        &self,
        origin: Vector3<f32>,
        direction: Vector3<f32>,
        nodes: &[&Node],
        ignore: &str,
    ) -> Option<RayHit> {
        self.hits(origin, direction.normalize(), nodes, |name| name != ignore)
            .into_iter()
            .next()
    }

    fn hits<F>(
        &self,
        origin: Vector3<f32>,
        direction: Vector3<f32>,
        nodes: &[&Node],
        filter: F,
    ) -> Vec<RayHit>
    where
        F: Fn(&str) -> bool,
    {
        let mut hits: Vec<RayHit> = nodes
            .iter()
            .filter(|node| filter(&node.name))
            .filter_map(|node| {
                let transform = node.get_component::<Transform>()?;
                let collider = node.get_component::<Collider>()?;

                let half_extents = Vector3::new(
                    collider.shape.half_extents().x * transform.scale.x,
                    collider.shape.half_extents().y * transform.scale.y,
                    collider.shape.half_extents().z * transform.scale.z,
                );

                let center =
                    transform.position + rotate_vector(transform.rotation, collider.offset);

                let axes = [
                    rotate_vector(transform.rotation, Vector3::new(1.0, 0.0, 0.0)),
                    rotate_vector(transform.rotation, Vector3::new(0.0, 1.0, 0.0)),
                    rotate_vector(transform.rotation, Vector3::new(0.0, 0.0, 1.0)),
                ];

                let (t, face) = intersect_obb(
                    origin,
                    direction,
                    self.max_distance,
                    center,
                    &axes,
                    half_extents,
                )?;

                let point = origin + direction * t;
                let normal = axes[face.axis] * face.sign;

                Some(RayHit {
                    node_name: node.name.clone(),
                    point,
                    normal,
                    distance: t,
                    is_static: collider.is_static,
                    is_area: collider.is_area,
                })
            })
            .collect();

        hits.sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap());
        hits
    }
}

struct HitFace {
    axis: usize,
    sign: f32,
}

fn intersect_obb(
    origin: Vector3<f32>,
    direction: Vector3<f32>,
    max_distance: f32,
    center: Vector3<f32>,
    axes: &[Vector3<f32>; 3],
    half: Vector3<f32>,
) -> Option<(f32, HitFace)> {
    let delta = origin - center;

    let half_diagonal = half.magnitude();
    if delta.dot(direction) > half_diagonal {
        return None;
    }

    let half_arr = [half.x, half.y, half.z];

    let mut t_min = f32::NEG_INFINITY;
    let mut t_max = f32::INFINITY;
    let mut hit_axis = 0usize;
    let mut hit_sign = 1.0f32;

    for i in 0..3 {
        let e = delta.dot(axes[i]);
        let f = direction.dot(axes[i]);

        if f.abs() > 1e-8 {
            let inv_f = 1.0 / f;
            let t1 = (-half_arr[i] - e) * inv_f;
            let t2 = (half_arr[i] - e) * inv_f;

            let (t_near_slab, t_far_slab) = if t1 < t2 { (t1, t2) } else { (t2, t1) };

            if t_near_slab > t_min {
                t_min = t_near_slab;
                hit_axis = i;
                // Ray travelling in +axis direction enters the -face, so normal is -axis
                hit_sign = -f.signum();
            }
            t_max = t_max.min(t_far_slab);

            if t_min > t_max {
                return None;
            }
        } else {
            if e.abs() > half_arr[i] {
                return None;
            }
        }
    }

    // Box entirely behind ray origin
    if t_max < 0.0 {
        return None;
    }

    // Ray started inside box — no hit for picking
    if t_min < 0.0 {
        return None;
    }

    // Beyond max distance
    if t_min > max_distance {
        return None;
    }

    Some((
        t_min,
        HitFace {
            axis: hit_axis,
            sign: hit_sign,
        },
    ))
}

fn rotate_vector(q: cgmath::Quaternion<f32>, v: Vector3<f32>) -> Vector3<f32> {
    let qv = Vector3::new(q.v.x, q.v.y, q.v.z);
    let t = qv.cross(v) * 2.0;
    v + t * q.s + qv.cross(t)
}

pub fn ray_from_mouse(
    mouse_x: f32,
    mouse_y: f32,
    viewport_width: f32,
    viewport_height: f32,
    projection: Matrix4<f32>,
    camera_position: Vector3<f32>,
    camera_rotation: Quaternion<f32>,
) -> (Vector3<f32>, Vector3<f32>) {
    let ndc_x = (mouse_x / viewport_width) * 2.0 - 1.0;
    let ndc_y = (mouse_y / viewport_height) * 2.0 - 1.0;

    let inv_proj = projection.invert().unwrap();

    let near_clip = Vector4::new(ndc_x, ndc_y, -1.0, 1.0);
    let near_view = inv_proj * near_clip;
    let near_view = near_view / near_view.w;

    let far_clip = Vector4::new(ndc_x, ndc_y, 1.0, 1.0);
    let far_view = inv_proj * far_clip;
    let far_view = far_view / far_view.w;

    let view_dir = Vector3::new(
        far_view.x - near_view.x,
        far_view.y - near_view.y,
        far_view.z - near_view.z,
    )
    .normalize();

    let world_dir = rotate_vector(camera_rotation, view_dir).normalize();

    (camera_position, world_dir)
}

pub fn pick(
    mouse_x: f32,
    mouse_y: f32,
    viewport_width: f32,
    viewport_height: f32,
    projection: Matrix4<f32>,
    camera_position: Vector3<f32>,
    camera_rotation: Quaternion<f32>,
    nodes: &[&Node],
    ignore: &str,
) -> Option<RayHit> {
    let (origin, direction) = ray_from_mouse(
        mouse_x,
        mouse_y,
        viewport_width,
        viewport_height,
        projection,
        camera_position,
        camera_rotation,
    );

    let ray = Raycast::new(direction, f32::MAX);
    ray.cast_from(origin, direction, nodes, ignore)
}
