//! An image-backed texture mapping

use image::{DynamicImage, GenericImageView, Pixel};

use crate::{color::Color, textures::Texture};

/// An image-based texture
#[derive(Debug)]
pub struct ImageMap {
    /// The image buffer used as the texture
    image: DynamicImage,
}

impl ImageMap {
    /// Creates a new [ImageMap]
    ///
    /// Loads the image located at `file_path`:
    /// * if successful, holds the decoded [image::RgbImage]
    /// * on error, holds a default "missing" texture
    ///
    /// Missing texture sourced from [The GMod fandom wiki](https://gmod.fandom.com/wiki/Missing_textures),
    /// available under CC-BY-SA
    pub fn new(file_path: std::path::PathBuf) -> Self {
        match image::open(&file_path) {
            Ok(image) => Self { image },
            Err(why) => {
                eprintln!("Failed to open {file_path:?}: {why}");
                Self::default()
            }
        }
    }
}

impl Default for ImageMap {
    fn default() -> Self {
        match image::load_from_memory(include_bytes!("../../resources/default.png")) {
            Ok(image) => Self { image },
            Err(_) => unreachable!("We should have access to the default image"),
        }
    }
}

impl Texture for ImageMap {
    fn color(&self, u: f32, v: f32, _point: glam::Vec3A) -> Color {
        let u = u.clamp(0.0, 1.0);
        let v = 1.0 - v.clamp(0.0, 1.0);

        let i = u * self.image.width() as f32;
        let j = v * self.image.height() as f32;

        let i = (i as u32).clamp(0, self.image.width() - 1);
        let j = (j as u32).clamp(0, self.image.height() - 1);

        let pixel = self.image.get_pixel(i, j).to_rgb();
        Color::from(pixel)
    }
}
