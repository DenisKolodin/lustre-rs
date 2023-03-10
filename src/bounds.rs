//! Implementation of bounding volumes

use glam::Vec3A;

use crate::{ray::Ray, utils::Axis};

/// An axis aligned bounding box
///
/// Many of the the methods are adapted from [pbrt 3rd edition](https://pbr-book.org/3ed-2018/Geometry_and_Transformations/Bounding_Boxes)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoundingBox {
    /// Minimum coordinates for each dimension
    pub min: Vec3A,
    /// Maximum coordinates for each dimension
    pub max: Vec3A,
}

// #[allow(dead_code)]
impl BoundingBox {
    /// Creates a new Axis aligned bounding box
    pub fn new(p0: Vec3A, p1: Vec3A) -> Self {
        Self {
            min: p0.min(p1),
            max: p0.max(p1),
        }
    }

    /// Creates a new BoundingBox without input validation
    pub fn new_unchecked(min: Vec3A, max: Vec3A) -> Self {
        Self { min, max }
    }

    /// Returns whether or not the ray hits this bounding box.
    ///
    /// Checks for slab intersection in each of the 3 dimensions.
    pub fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> bool {
        let inverse_dir = ray.direction.recip();
        let diff0 = self.min - ray.origin;
        let diff1 = self.max - ray.origin;

        // Check for slab intersection in each dimension
        for axis_idx in Axis::AXES {
            let inverse_dir = inverse_dir[axis_idx];
            let t0 = diff0[axis_idx] * inverse_dir;
            let t1 = diff1[axis_idx] * inverse_dir;

            // swap if inverted
            let (t0, t1) = if inverse_dir < 0.0 {
                (t1, t0)
            } else {
                (t0, t1)
            };

            let t_near = t0.max(t_min);
            let t_far = t1.min(t_max);
            if t_far <= t_near {
                return false;
            }
        }

        true
    }

    /// Returns whether or not the ray hits this bounding box, using the ray's precomputed inverse direction.
    ///
    /// Similar to [BoundingBox::hit], this function checks for slab intersection in each of the 3 dimensions.
    /// In addition, this function minimizes branching by using [f32::min] and [f32::max] intrinsics.
    /// Based on the branchless bounding box intersection codes from [Tavian Barnes' blog](https://tavianator.com/2022/ray_box_boundary.html)
    pub fn hit_with_inv(&self, ray: &Ray, ray_dir_inv: Vec3A, t_min: f32, t_max: f32) -> bool {
        let diff0 = self.min - ray.origin;
        let diff1 = self.max - ray.origin;

        let mut t_near = t_min;
        let mut t_far = t_max;

        // Check for slab intersection in each dimension
        for axis_idx in Axis::AXES {
            let inverse_dir = ray_dir_inv[axis_idx];
            let t0 = diff0[axis_idx] * inverse_dir;
            let t1 = diff1[axis_idx] * inverse_dir;

            // these set of comparison allow for corner and parallel intersection checks
            t_near = f32::min(t_near.max(t0), t_near.max(t1));
            t_far = f32::max(t_far.min(t0), t_far.min(t1));
        }

        t_near <= t_far
    }

    /// Returns a bounding box enclosing this and the other box.
    ///
    /// In other words, combines the two boxes by taking:
    /// * the minimums of the two boxes' min members
    /// * the maximums of the two boxes' max members
    pub fn union(&self, other: BoundingBox) -> BoundingBox {
        Self {
            min: self.min.min(other.min),
            max: self.max.max(other.max),
        }
    }

    /// Returns a bounding box enclosing this and the given point
    pub fn add_point(&self, point: Vec3A) -> BoundingBox {
        Self {
            min: self.min.min(point),
            max: self.max.max(point),
        }
    }

    /// Returns the vector along the box diagonal (from min point to max point)
    pub fn diagonal(&self) -> Vec3A {
        debug_assert!(self.min.cmplt(self.max).all());
        self.max - self.min
    }

    /// Returns the total surface area of the bounding box
    pub fn surface_area(&self) -> f32 {
        let d = self.diagonal();
        2.0 * (d.x * d.y + d.x * d.z + d.y * d.z)
    }

    /// Returns the total volume of the bounding box
    pub fn volume(&self) -> f32 {
        let d = self.diagonal();
        d.x * d.y * d.z
    }

    /// Returns the longest [Axis] of the bounding box
    pub fn longest_axis(&self) -> Axis {
        let d = self.diagonal();
        if d.x > d.y && d.x > d.z {
            Axis::X
        } else if d.y > d.z {
            Axis::Y
        } else {
            Axis::Z
        }
    }

    /// Returns the position within the bounding box, relative to the corners of the bounding box
    pub fn offset(&self, point: Vec3A) -> Vec3A {
        (point - self.min) / self.diagonal()
    }

    /// Returns the point that lies at the center of the bounding box
    pub fn centroid(&self) -> Vec3A {
        0.5 * self.min + 0.5 * self.max
    }

    /// Returns whether or not this bounding box overlaps the other
    pub fn overlaps(&self, other: &Self) -> bool {
        self.max.cmpge(other.min).all() && self.min.cmple(other.max).all()
    }

    /// Returns whether or not the given point is inside this bounding box
    pub fn inside(&self, point: Vec3A) -> bool {
        self.max.cmpge(point).all() && self.min.cmple(point).all()
    }
}

impl Default for BoundingBox {
    fn default() -> Self {
        Self {
            min: Vec3A::splat(f32::MAX),
            max: Vec3A::splat(f32::MIN),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_union() {
        let def = BoundingBox::default();
        let zeroes = BoundingBox::new(Vec3A::ZERO, Vec3A::ZERO);
        assert_eq!(
            def.union(zeroes),
            zeroes,
            "The union of the default bbox with another bbox should be equal to the other bbox"
        )
    }
}
