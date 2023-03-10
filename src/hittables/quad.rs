//! Quadrilateral implementation

use std::sync::Arc;

use glam::{Vec2, Vec3A};

use crate::{
    bounds::BoundingBox,
    hittables::{HitRecord, Hittable},
    material::Material,
};

/// A quadrilateral defined by four points in space
///
/// Based on Inigo Quilez's quad intersector:
/// * [Intersection ShaderToy example](https://www.shadertoy.com/view/XtlBDs)
/// * [Surface Ooords ShaderToy example](https://www.shadertoy.com/view/lsBSDm)
#[derive(Debug)]
pub struct Quad {
    p0: Vec3A,
    p1: Vec3A,
    p2: Vec3A,
    p3: Vec3A,
    pub material: Arc<Material>,
}

impl Quad {
    /// axes indices used during 2d projection.
    const AXIS_IDXS: [usize; 4] = [1, 2, 0, 1];
    // 0----3
    // |    |
    // |    |
    // 1----2
    /// Creates a new Quad.
    pub fn new(p0: Vec3A, p1: Vec3A, p2: Vec3A, p3: Vec3A, m: &Arc<Material>) -> Self {
        Self {
            p0,
            p1,
            p2,
            p3,
            material: Arc::clone(m),
        }
    }

    pub fn from_bounds_k(
        a_min: f32,
        a_max: f32,
        b_min: f32,
        b_max: f32,
        k: f32,
        axis: usize,
        m: &Arc<Material>,
    ) -> Self {
        let (p0, p1, p2, p3) = match axis {
            0 => {
                let p0 = Vec3A::new(k, a_min, b_min);
                let p1 = Vec3A::new(k, a_max, b_min);
                let p2 = Vec3A::new(k, a_max, b_max);
                let p3 = Vec3A::new(k, a_min, b_max);
                (p0, p1, p2, p3)
            }
            1 => {
                let p0 = Vec3A::new(a_min, k, b_min);
                let p1 = Vec3A::new(a_max, k, b_min);
                let p2 = Vec3A::new(a_max, k, b_max);
                let p3 = Vec3A::new(a_min, k, b_max);
                (p0, p1, p2, p3)
            }
            2 => {
                let p0 = Vec3A::new(a_min, b_min, k);
                let p1 = Vec3A::new(a_max, b_min, k);
                let p2 = Vec3A::new(a_max, b_max, k);
                let p3 = Vec3A::new(a_min, b_max, k);
                (p0, p1, p2, p3)
            }
            _ => panic!("Invalid axis index"),
        };

        Self::new(p0, p1, p2, p3, m)
    }

    /// Creates a new axis-aligned Quad based on 2 points on a plane + the plane's k value.
    ///
    /// Requires one dimension in each point to be zero-ed out to work.
    pub fn from_two_points_z(p_min: Vec3A, p_max: Vec3A, k: f32, m: &Arc<Material>) -> Self {
        let (x_min, y_min, z_min) = p_min.into();
        let (x_max, y_max, z_max) = p_max.into();

        // Check which dimension to use z value in
        let (p0, p1, p2, p3) = if x_min == x_max && x_min == 0.0 {
            let p0 = Vec3A::new(k, y_min, z_min);
            let p1 = Vec3A::new(k, y_max, z_min);
            let p2 = Vec3A::new(k, y_max, z_max);
            let p3 = Vec3A::new(k, y_min, z_max);
            (p0, p1, p2, p3)
        } else if y_min == y_max && y_min == 0.0 {
            let p0 = Vec3A::new(x_min, k, z_min);
            let p1 = Vec3A::new(x_max, k, z_min);
            let p2 = Vec3A::new(x_max, k, z_max);
            let p3 = Vec3A::new(x_min, k, z_max);
            (p0, p1, p2, p3)
        } else if z_min == z_max && z_min == 0.0 {
            let p0 = Vec3A::new(x_min, y_min, k);
            let p1 = Vec3A::new(x_max, y_min, k);
            let p2 = Vec3A::new(x_max, y_max, k);
            let p3 = Vec3A::new(x_min, y_max, k);
            (p0, p1, p2, p3)
        } else {
            panic!("Points are not zero in the same dimension! {p_min} vs {p_max}");
        };

        Self::new(p0, p1, p2, p3, m)
    }

    fn cross(a: Vec2, b: Vec2) -> f32 {
        a.x * b.y - a.y * b.x
    }
}

impl Hittable for Quad {
    fn bounding_box(&self, _time0: f32, _time1: f32) -> Option<BoundingBox> {
        // its an aabb :/
        let min = self.p0.min(self.p1).min(self.p2).min(self.p3) - 0.0001;
        let max = self.p0.max(self.p1).max(self.p2).max(self.p3) + 0.0001;
        Some(BoundingBox::new_unchecked(min, max))
    }

    fn hit(&self, ray: &crate::ray::Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        // see https://www.shadertoy.com/view/XtlBDs
        // 0--b--3
        // |\
        // a c
        // |  \
        // 1    2
        let a = self.p1 - self.p0;
        let b = self.p3 - self.p0;
        let c = self.p2 - self.p0;
        let p = ray.origin - self.p0;

        // plane intersection
        let plane_normal = a.cross(b);
        let t = -p.dot(plane_normal) / ray.direction.dot(plane_normal);

        // check against t bounds
        if t < t_min || t > t_max {
            return None;
        }

        let point = p + ray.direction * t;

        // Projecting to 2D ("plane space")
        let abs_normal = plane_normal.abs();
        let axis_idx = if abs_normal.x > abs_normal.y && abs_normal.x > abs_normal.z {
            0
        } else if abs_normal.y > abs_normal.z {
            1
        } else {
            2
        };

        let id_u = Self::AXIS_IDXS[axis_idx];
        let id_v = Self::AXIS_IDXS[axis_idx + 1];

        // projection
        let kp = Vec2::new(point[id_u], point[id_v]);
        let ka = Vec2::new(a[id_u], a[id_v]);
        let kb = Vec2::new(b[id_u], b[id_v]);
        let kc = Vec2::new(c[id_u], c[id_v]);

        // barycentric coords
        let kg = kc - kb - ka;

        let k0 = Self::cross(kp, kb);
        let k1 = Self::cross(kp, kg) - plane_normal[axis_idx];
        let k2 = Self::cross(kc - kb, ka);

        let (u, v) = if k2.abs() < 1e-5 {
            // if edges are parallel, solution is a linear eq
            let u = Self::cross(kp, ka) / k1;
            let v = -k0 / k1;
            (u, v)
        } else {
            // otherwise, solution is quadratic eq
            let w = k1 * k1 - 4.0 * k0 * k2;
            if w < 0.0 {
                return None;
            }

            let w = w.sqrt();
            let ik2 = (2.0 * k2).recip();
            let mut v = (-k1 - w) * ik2;
            if !(0.0..=1.0).contains(&v) {
                v = (-k1 + w) * ik2;
            }

            let u = (kp.x - ka.x * v) / (kb.x + kg.x * v);

            (u, v)
        };

        if !(0.0..=1.0).contains(&u) || !(0.0..=1.0).contains(&v) {
            return None;
        }

        let normal = abs_normal.normalize();
        let mut rec = HitRecord {
            point: ray.at(t),
            normal,
            material: Arc::clone(&self.material),
            t,
            u,
            v,
            front_face: true,
        };
        rec.set_face_normal(ray, normal);

        Some(rec)
    }
}
