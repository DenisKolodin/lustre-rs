//! Render an image given a [Camera] and a [Hittable].

use glam::Vec3A;
use rand::{rngs::SmallRng, Rng, SeedableRng};

#[cfg(feature = "parallel")]
use {indicatif::ParallelProgressIterator, rayon::prelude::*};

#[cfg(not(feature = "parallel"))]
use indicatif::ProgressIterator;

use crate::{camera::Camera, color::Color, hittables::Hittable, utils::progress::get_progressbar};

/// Image Renderer storing scene context values such as image dimensions and samples per pixel
#[derive(Debug, Clone, Copy)]
pub struct Renderer {
    image_width: u32,
    image_height: u32,
    samples_per_pixel: u32,
    bounce_depth: u16,
}

impl Renderer {
    /// Creates a new [Renderer].
    pub fn new(
        image_width: u32,
        image_height: u32,
        samples_per_pixel: u32,
        bounce_depth: u16,
    ) -> Self {
        Self {
            image_width,
            image_height,
            samples_per_pixel,
            bounce_depth,
        }
    }

    /// Calculates the total color value of the pixel at image coordinates (`x`, `y`)
    ///
    /// Uses the provided [Camera] to translate the image coordinates
    /// to world space coordinates, then computes the color value
    #[inline]
    fn compute_pixel_v(
        &self,
        cam: &Camera,
        world: &impl Hittable,
        x: u32,
        y: u32,
        rng: &mut impl Rng,
    ) -> Vec3A {
        // convert buffer indices to viewport coordinates
        let offset_u: f32 = rng.gen();
        let offset_v: f32 = rng.gen();
        let u = (x as f32 + offset_u) / (self.image_width - 1) as f32;
        let v = ((self.image_height - y) as f32 + offset_v) / (self.image_height - 1) as f32;

        // trace ray
        let contrib = cam
            .get_ray(u, v, rng)
            .shade(world, self.bounce_depth, cam.bg_color, rng);
        Vec3A::from(contrib)
    }

    /// Generates an image from the given scene.
    ///
    /// A scene consists of a [Camera] and some [Hittable].
    /// This functions outputs its progress to the commandline.
    pub fn render_scene(&self, (cam, world): (Camera, impl Hittable)) -> image::RgbImage {
        let progress_bar = get_progressbar((self.image_height * self.image_width) as u64)
            .with_prefix("Generating pixels");

        // Allocate image buffer
        let mut img_buf: image::RgbImage =
            image::ImageBuffer::new(self.image_width, self.image_height);

        // Generate image
        #[cfg(feature = "parallel")]
        img_buf
            .enumerate_pixels_mut()
            .par_bridge()
            .progress_with(progress_bar)
            .for_each(|(x, y, pixel)| {
                // map reduce N samples into single Vec3A
                let mut color_v: Vec3A = (0..self.samples_per_pixel)
                    .into_par_iter()
                    .map_init(
                        // from_rng(...) gives Result, can assume it won't fail
                        || SmallRng::from_rng(&mut rand::thread_rng()).unwrap(),
                        // current sample # doesn't matter, ignore
                        |rng, _| self.compute_pixel_v(&cam, &world, x, y, rng),
                    )
                    .sum();

                // Account for number of samples
                color_v /= self.samples_per_pixel as f32;

                // "gamma" correction
                color_v = color_v.powf(0.5); // sqrt

                // modify pixel with generated color value
                *pixel = image::Rgb::<u8>::from(Color::new(color_v));
            });
        #[cfg(not(feature = "parallel"))]
        img_buf
            .enumerate_pixels_mut()
            .progress_with(progress_bar)
            .for_each(|(x, y, pixel)| {
                // map reduce N samples into single Vec3A
                let mut color_v: Vec3A = (0..self.samples_per_pixel)
                    .map(
                        // current sample # doesn't matter, ignore
                        |_| {
                            let rng = &mut SmallRng::from_rng(&mut rand::thread_rng()).unwrap();
                            self.compute_pixel_v(&cam, &world, x, y, rng)
                        },
                    )
                    .sum();

                // Account for number of samples
                color_v /= self.samples_per_pixel as f32;

                // "gamma" correction
                color_v = color_v.powf(0.5); // sqrt

                // modify pixel with generated color value
                *pixel = image::Rgb::<u8>::from(Color::new(color_v));
            });

        img_buf
    }
}
