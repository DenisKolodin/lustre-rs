//! Contains description of what it means to intersect something,
//! as well as what's returned on intersection

use std::sync::Arc;

use glam::Vec3A;

use crate::{bounds::BoundingBox, material::Material, ray::Ray};

pub mod list;
pub mod quad;
pub mod quadbox;
pub mod sphere;
pub mod transform;
pub mod volume;

pub use list::*;
pub use quad::*;
pub use quadbox::*;
pub use sphere::*;
pub use transform::*;
pub use volume::*;

/// Defines a set of data returned upon a successful intersection
#[derive(Debug)]
pub struct HitRecord {
    /// Point of intersection in 3D space
    pub point: Vec3A,
    /// Surface normal off the point of intersection
    pub normal: Vec3A,
    /// Material of the intersected object
    pub material: Arc<Material>,
    /// distance from the origin to the point of intersection
    pub t: f32,
    /// u coordinate of surface of point of intersection
    pub u: f32,
    /// v coordinate of surface of point of intersection
    pub v: f32,
    /// Whether or not the ray hit the object's inside or outside face
    pub front_face: bool,
}

impl HitRecord {
    pub fn set_face_normal(&mut self, ray: &Ray, outward_n: Vec3A) {
        if ray.direction.dot(outward_n) < 0.0 {
            // ray outside sphere
            self.front_face = true;
            self.normal = outward_n;
        } else {
            // ray inside sphere
            self.front_face = false;
            self.normal = -outward_n;
        }
    }
}

impl PartialOrd for HitRecord {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.t.partial_cmp(&other.t)
    }
}

impl PartialEq for HitRecord {
    fn eq(&self, other: &Self) -> bool {
        self.t.eq(&other.t)
    }
}

/// Describes the behavior of objects that support intersection
pub trait Hittable: Send + Sync {
    /// Intersects the given ray with the object
    ///
    /// Returns a `Some(HitRecord)` if successful, otherwise `None`
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord>;

    /// Returns the axis aligned bounding box for the object
    ///
    /// Returns a `Some(Aabb)` if the object has a bounding box (like spheres), otherwise `None` (like planes)
    fn bounding_box(&self, time0: f32, time1: f32) -> Option<BoundingBox>;

    fn wrap(self) -> Arc<Self>
    where
        Self: Sized,
    {
        Arc::new(self)
    }
}

impl Hittable for Arc<dyn Hittable> {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        self.as_ref().hit(ray, t_min, t_max)
    }

    fn bounding_box(&self, time0: f32, time1: f32) -> Option<BoundingBox> {
        self.as_ref().bounding_box(time0, time1)
    }
}
