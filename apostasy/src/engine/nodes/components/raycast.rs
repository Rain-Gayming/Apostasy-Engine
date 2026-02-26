use cgmath::{InnerSpace, Rotation, Vector3};
use serde::{Deserialize, Serialize};

use crate::engine::editor::inspectable::Inspectable;
use crate::engine::nodes::{
    World,
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

    /// Fires a ray from `transform.position` along `self.direction` rotated by `transform.rotation`.
    pub fn cast(&self, transform: &Transform, world: &World, ignore: &str) -> Option<RayHit> {
        let world_dir = transform.rotation.rotate_vector(self.direction).normalize();
        self.hits(transform.position, world_dir, world, |name| name != ignore)
            .into_iter()
            .next()
    }

    /// Same as `cast` but returns every hit sorted nearest-first.
    pub fn cast_all(&self, transform: &Transform, world: &World, ignore: &str) -> Vec<RayHit> {
        let world_dir = transform.rotation.rotate_vector(self.direction).normalize();
        self.hits(transform.position, world_dir, world, |name| name != ignore)
    }

    /// Same as `cast` but ignores multiple nodes by name.
    pub fn cast_ignore_many(
        &self,
        transform: &Transform,
        world: &World,
        ignore: &[&str],
    ) -> Option<RayHit> {
        let world_dir = transform.rotation.rotate_vector(self.direction).normalize();
        self.hits(transform.position, world_dir, world, |name| {
            !ignore.contains(&name)
        })
        .into_iter()
        .next()
    }

    // ── Internal ──────────────────────────────────────────────────────────────

    fn hits<F>(
        &self,
        origin: Vector3<f32>,
        direction: Vector3<f32>,
        world: &World,
        filter: F,
    ) -> Vec<RayHit>
    where
        F: Fn(&str) -> bool,
    {
        let mut hits: Vec<RayHit> = world
            .get_all_nodes()
            .into_iter()
            .filter(|node| filter(&node.name))
            .filter_map(|node| {
                let transform = node.get_component::<Transform>()?;
                let collider = node.get_component::<Collider>()?;
                let center = transform.position + collider.offset;
                let t = intersect_aabb(
                    origin,
                    direction,
                    self.max_distance,
                    collider.world_min(center),
                    collider.world_max(center),
                )?;
                let point = origin + direction * t;
                let normal = surface_normal(point, center, collider.half_extents);
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

// ─── Slab intersection (Kay–Kajiya) ──────────────────────────────────────────

fn intersect_aabb(
    origin: Vector3<f32>,
    direction: Vector3<f32>,
    max_distance: f32,
    min: Vector3<f32>,
    max: Vector3<f32>,
) -> Option<f32> {
    let inv = Vector3::new(
        safe_recip(direction.x),
        safe_recip(direction.y),
        safe_recip(direction.z),
    );

    let t1 = (min.x - origin.x) * inv.x;
    let t2 = (max.x - origin.x) * inv.x;
    let t3 = (min.y - origin.y) * inv.y;
    let t4 = (max.y - origin.y) * inv.y;
    let t5 = (min.z - origin.z) * inv.z;
    let t6 = (max.z - origin.z) * inv.z;

    let t_near = t1.min(t2).max(t3.min(t4)).max(t5.min(t6));
    let t_far = t1.max(t2).min(t3.max(t4)).min(t5.max(t6));

    if t_far < 0.0 || t_near > t_far {
        return None;
    }

    let t = if t_near < 0.0 { t_far } else { t_near };
    if t > max_distance { None } else { Some(t) }
}

fn surface_normal(
    hit_point: Vector3<f32>,
    center: Vector3<f32>,
    half_extents: Vector3<f32>,
) -> Vector3<f32> {
    let local = hit_point - center;
    let dx = (local.x.abs() - half_extents.x).abs();
    let dy = (local.y.abs() - half_extents.y).abs();
    let dz = (local.z.abs() - half_extents.z).abs();

    if dx < dy && dx < dz {
        Vector3::new(local.x.signum(), 0.0, 0.0)
    } else if dy < dx && dy < dz {
        Vector3::new(0.0, local.y.signum(), 0.0)
    } else {
        Vector3::new(0.0, 0.0, local.z.signum())
    }
}

#[inline]
fn safe_recip(x: f32) -> f32 {
    if x.abs() > 1e-8 { 1.0 / x } else { f32::MAX }
}
