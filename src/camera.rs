//! Implementation of a camera
//!
//! # Features
//! * positionable and orientable - Using the `look_from`, `look_at`, and `view_up` triplet of vectors
//! * resizable film - Using `aspect_ratio`
//! * depth of field (aka defocus blur) - Using the `aperture` and `focus_dist` data

use std::ops::Range;

use glam::Vec3A;
use rand::Rng;

use crate::{
    color::{colors, Color},
    ray::Ray,
    utils::random::rand_vec3_in_unit_disk,
};

/// A Camera that generates rays
#[derive(Debug, Clone, Copy)]
pub struct Camera {
    /// Camera position in space
    origin: Vec3A,
    /// Position of the viewport's lower left corner
    ll_corner: Vec3A,
    /// Horizontal 'size' of the viewport
    horizontal: Vec3A,
    /// Vertical 'size' of the viewport
    vertical: Vec3A,
    /// Orthonormal base 1
    u: Vec3A,
    /// Orthonormal base 2
    v: Vec3A,
    /// Orthonormal base 3, works like focal length
    #[allow(dead_code/* , reason = "Surely there's gonna be a use for this right?" */)]
    w: Vec3A,
    /// Radius of the approximated camera lens
    lens_radius: f32,
    /// Shutter open time,
    pub shutter_open_time: f32,
    /// Shutter close time
    pub shutter_close_time: f32,
    /// Background color
    pub bg_color: Color,
    /// Aspect ratio
    pub aspect_ratio: f32,
}

impl Camera {
    /// Creates a new Camera
    ///
    /// # Arguments
    /// * look_from - A [Vec3A] holding the position of the camera
    /// * look_at - A [Vec3A] holding the eye direction of the camera
    /// * view_up - A [Vec3A] holding the "up" direction of the camera
    /// * vert_fov - The vertical field of view
    /// * aspect_ratio - The aspect ratio of the viewport
    /// * aperture - How "big" the approximated lens is
    /// * focus_dist - The distance to the plane in space where objects are "in focus"
    pub fn new(
        look_from: Vec3A,
        look_at: Vec3A,
        view_up: Vec3A,
        vert_fov: f32,
        aspect_ratio: f32,
        aperture: f32,
        focus_dist: f32,
        shutter_time: Range<f32>,
        bg_color: Color,
    ) -> Self {
        // Set up viewport
        let theta = vert_fov.to_radians();
        let viewport_h = 2.0 * (theta / 2.0).tan();
        let viewport_w = aspect_ratio * viewport_h;

        // Set up position
        let w = (look_from - look_at).normalize();
        let u = view_up.cross(w).normalize();
        let v = w.cross(u);

        let origin = look_from;
        let horizontal = viewport_w * focus_dist * u;
        let vertical = viewport_h * focus_dist * v;
        let ll_corner = origin - horizontal / 2.0 - vertical / 2.0 - focus_dist * w;

        let lens_radius = aperture / 2.0;
        Self {
            origin,
            ll_corner,
            horizontal,
            vertical,
            u,
            v,
            w,
            lens_radius,
            shutter_open_time: shutter_time.start,
            shutter_close_time: shutter_time.end,
            bg_color,
            aspect_ratio,
        }
    }

    /// Returns a ray from the camera for the normalized pixel (u,v)
    pub fn get_ray(&self, u: f32, v: f32, rng: &mut impl Rng) -> Ray {
        let rd = self.lens_radius * rand_vec3_in_unit_disk(rng);
        let offset = self.u * rd.x + self.v * rd.y;
        Ray {
            origin: self.origin + offset,
            direction: self.ll_corner + u * self.horizontal + v * self.vertical
                - self.origin
                - offset,
            time: rng.gen_range(self.shutter_open_time..self.shutter_close_time),
        }
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::new(
            Vec3A::ZERO,
            -Vec3A::Z,
            Vec3A::Y,
            20.0,
            16.0 / 9.0,
            0.0,
            10.0,
            0.0..1.0,
            colors::BLACK,
        )
    }
}
