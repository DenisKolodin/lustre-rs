use std::rc::Rc;

use crate::{bounds::Aabb, ray::Ray};

use super::{HitRecord, Hittable};

/// Type alias for a vector of objects implementing [Hittable]
pub type HittableList = Vec<Rc<dyn Hittable>>;

impl Hittable for HittableList {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let mut rec = None;
        let mut t_closest = t_max;

        for hittable in self.iter() {
            let hit_result = hittable.hit(ray, t_min, t_closest);
            if let Some(HitRecord { t, .. }) = hit_result {
                t_closest = t;
                rec = hit_result;
            }
        }
        rec
    }

    fn bounding_box(&self, time0: f32, time1: f32) -> Option<Aabb> {
        if self.is_empty() {
            return None;
        }

        // applies the bounding_box fn to all hittables,
        // filters out those returning `None`,
        // and reduces to a single bounding box through
        // repeated applications of the union fn.
        self.iter()
            .filter_map(|hittable| hittable.bounding_box(time0, time1))
            .reduce(|acc, bbox| acc.union(&bbox))
    }
}