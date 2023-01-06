//! Hittable Instance that tranforms the ontained hittable

use std::sync::Arc;

use glam::{Affine3A, Vec3};

use crate::{
    bounds::BoundingBox,
    hittables::{HitRecord, Hittable},
};

/// A hittable undergoes a transform before and after being hit.
pub struct Transform {
    transform: Affine3A,
    inv_transform: Affine3A,
    object: Arc<dyn Hittable>,
}

#[allow(dead_code/* , reason = "Exposing Affine3A constructors" */)]
impl Transform {
    // creators

    /// Creates an affine transform that does not transform the underlying object
    pub fn new(o: &Arc<dyn Hittable>) -> Self {
        let transform = Affine3A::IDENTITY;
        Self {
            transform,
            inv_transform: transform.inverse(),
            object: Arc::clone(o),
        }
    }

    /// Creates an affine transform that changes the size of the object.
    pub fn from_scale_factor(scale: Vec3, o: &Arc<dyn Hittable>) -> Self {
        let transform = Affine3A::from_scale(scale);
        Self {
            transform,
            inv_transform: transform.inverse(),
            object: Arc::clone(o),
        }
    }

    /// Creates an affine transform containing a 3D rotation around an `axis`, of `angle` (in radians).
    pub fn from_axis_angle(axis: Vec3, angle: f32, o: &Arc<dyn Hittable>) -> Self {
        let transform = Affine3A::from_axis_angle(axis, angle);
        Self {
            transform,
            inv_transform: transform.inverse(),
            object: Arc::clone(o),
        }
    }

    /// Creates an affine transform from the given 3D `translation`.
    pub fn from_translation(translation: Vec3, o: &Arc<dyn Hittable>) -> Self {
        let transform = Affine3A::from_translation(translation);
        Self {
            transform,
            inv_transform: transform.inverse(),
            object: Arc::clone(o),
        }
    }

    /// Creates a view transform using a camera position, a focal point, and an up direction.
    pub fn look_at(
        camera_pos: Vec3,
        focal_point: Vec3,
        up_dir: Vec3,
        o: &Arc<dyn Hittable>,
    ) -> Self {
        let transform = Affine3A::look_at_rh(camera_pos, focal_point, up_dir);
        Self {
            transform,
            inv_transform: transform.inverse(),
            object: Arc::clone(o),
        }
    }

    // builders

    /// Adds a scaling factor to the existing affine transform
    pub fn with_scale_factor(&mut self, scale: Vec3) -> &mut Self {
        self.transform = Affine3A::from_scale(scale) * self.transform;
        self
    }

    /// Adds a rotation based on the axis and angle (in radians) to the existing affine transform
    pub fn with_axis_angle_radians(&mut self, axis: Vec3, radians: f32) -> &mut Self {
        self.transform = Affine3A::from_axis_angle(axis, radians) * self.transform;
        self
    }

    /// Adds a rotation based on the axis and angle (in degrees) to the existing affine transform
    pub fn with_axis_angle_degrees(&mut self, axis: Vec3, degrees: f32) -> &mut Self {
        self.transform = Affine3A::from_axis_angle(axis, degrees.to_radians()) * self.transform;
        self
    }

    /// Adds a translation to the existing affine transform
    pub fn with_translation(&mut self, translation: Vec3) -> &mut Self {
        self.transform = Affine3A::from_translation(translation) * self.transform;
        self
    }

    pub fn finalize(&mut self) -> Self {
        Self {
            transform: self.transform,
            inv_transform: self.transform.inverse(),
            object: self.object.to_owned(),
        }
    }
}

impl Hittable for Transform {
    fn hit(&self, ray: &crate::ray::Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let transformed_ray = crate::ray::Ray::new(
            self.inv_transform.transform_point3a(ray.origin),
            self.inv_transform.transform_vector3a(ray.direction),
            ray.time,
        );

        match self.object.hit(&transformed_ray, t_min, t_max) {
            Some(rec) => {
                let mut transformed_rec = HitRecord {
                    point: self.transform.transform_point3a(rec.point),
                    normal: self.transform.transform_vector3a(rec.normal),
                    ..rec
                };
                transformed_rec.set_face_normal(&transformed_ray, rec.normal);
                Some(transformed_rec)
            }
            None => None,
        }
    }

    fn bounding_box(&self, time0: f32, time1: f32) -> Option<BoundingBox> {
        self.object
            .bounding_box(time0, time1)
            .map(|BoundingBox { min, max }| {
                let new_min = self.transform.transform_point3a(min);
                let new_max = self.transform.transform_point3a(max);
                BoundingBox::new(new_min, new_max)
            })
    }
}
