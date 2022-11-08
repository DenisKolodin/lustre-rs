//! A texture mapping alternating between two other Textures in a checkerboard fashion.

use crate::{color::Color, textures::Texture};
use std::sync::Arc;

/// A checkered texture alternating between two enclosed textures.
#[derive(Debug)]
pub struct Checkered<T, U>
where
    T: Texture,
    U: Texture,
{
    pub even: Arc<T>,
    pub odd: Arc<U>,
}

impl<T, U> Checkered<T, U>
where
    T: Texture + Sized,
    U: Texture + Sized,
{
    /// Creates a new checkered texture
    pub fn new(e: &Arc<T>, o: &Arc<U>) -> Self {
        Self {
            even: Arc::clone(e),
            odd: Arc::clone(o),
        }
    }
}

impl<T, U> Texture for Checkered<T, U>
where
    T: Texture,
    U: Texture,
{
    fn color(&self, u: f32, v: f32, point: glam::Vec3A) -> Color {
        let sin_x = (point * 10.0).x.sin();
        let sin_y = (point * 10.0).y.sin();
        let sin_z = (point * 10.0).z.sin();

        if sin_x * sin_y * sin_z < 0.0 {
            self.odd.color(u, v, point)
        } else {
            self.even.color(u, v, point)
        }
    }
}
