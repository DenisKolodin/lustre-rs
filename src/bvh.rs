//! Bounding Volume Hierarchy
#![allow(dead_code)]

use std::{cmp::Ordering, fmt::Debug, sync::Arc};

use crate::{
    bounds::BoundingBox,
    hittables::{HitRecord, Hittable, HittableList},
    ray::Ray,
    utils::{match_opts::match_opts, Axis},
};

/// A node in the BVH.
///
/// Holds the bounding box that contains the two [Hittable] children
pub struct BvhNode {
    /// left portion of the subhierarchy
    left: Arc<dyn Hittable>,
    /// right portion of the subhierarchy
    right: Arc<dyn Hittable>,
    /// AABB of the current hierarchy
    bbox: BoundingBox,
}

/// Compares two bounding boxes based on existence and then along the given axis
pub fn box_cmp(a: &Option<BoundingBox>, b: &Option<BoundingBox>, axis_idx: Axis) -> Ordering {
    match (a, b) {
        (None, None) => {
            panic!("box_cmp encountered two unbounded objects");
        }
        (None, Some(_)) => Ordering::Less,
        (Some(_), None) => Ordering::Greater,
        (Some(a_box), Some(b_box)) => a_box.min[axis_idx]
            .partial_cmp(&b_box.min[axis_idx])
            .expect("boxes contained extreme FP values"),
    }
}

impl BvhNode {
    /// Creates a new BvhNode
    pub fn new(hitlist: HittableList, time0: f32, time1: f32) -> Self {
        BvhNode::new_node(hitlist, time0, time1)
    }

    /// Implementation of `new`
    fn new_node(mut hitlist: HittableList, time0: f32, time1: f32) -> Self {
        assert!(!hitlist.is_empty(), "Given empty scene!");

        let span = hitlist.len();

        let (left, right) = match span {
            1 => (hitlist[0].clone(), hitlist[0].clone()),
            2 => (hitlist[0].clone(), hitlist[1].clone()),
            _ => {
                let axis_idx = hitlist.bounding_box(time0, time1).unwrap().longest_axis();

                hitlist.sort_by(|a, b| {
                    box_cmp(
                        &a.bounding_box(time0, time1),
                        &b.bounding_box(time0, time1),
                        axis_idx,
                    )
                });

                let (half0, half1) = hitlist.split_at_mut(span / 2);

                let left: Arc<dyn Hittable> =
                    BvhNode::new_node(half0.to_owned(), time0, time1).wrap();
                let right: Arc<dyn Hittable> =
                    BvhNode::new_node(half1.to_owned(), time0, time1).wrap();
                (left, right)
            }
        };

        let bbox = match (
            left.bounding_box(time0, time1),
            right.bounding_box(time0, time1),
        ) {
            (None, None) => {
                panic!("new_node encountered two unbounded objects");
            }
            (None, Some(b)) => b,
            (Some(a), None) => a,
            (Some(a), Some(b)) => a.union(b),
        };

        Self { left, right, bbox }
    }
}

impl Debug for BvhNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BvhNode {{{:?}}}", self.bbox)
    }
}

impl Hittable for BvhNode {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        if self.bbox.hit(ray, t_min, t_max) {
            let left_hit = self.left.hit(ray, t_min, t_max);

            let t_max = match &left_hit {
                Some(rec) => rec.t,
                None => t_max,
            };

            let right_hit = self.right.hit(ray, t_min, t_max);
            match_opts(left_hit, right_hit, |a, b| if a.t < b.t { a } else { b })
        } else {
            None
        }
    }

    fn bounding_box(&self, _time0: f32, _time1: f32) -> Option<BoundingBox> {
        Some(self.bbox)
    }
}
