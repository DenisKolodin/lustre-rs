//! Color and pixel output

use glam::Vec3A;

/// A RGB color.
///
/// Holds its value as a [Vec3A]
#[derive(Debug, Clone, Copy)]
pub struct Color {
    value: Vec3A,
}

impl Color {
    /// Creates a new Color
    pub fn new(value: Vec3A) -> Self {
        Self { value }
    }
}

impl From<Color> for Vec3A {
    fn from(color: Color) -> Self {
        color.value
    }
}

// The important stuff
impl From<Color> for image::Rgb<u8> {
    fn from(color: Color) -> Self {
        Self(
            color
                .value
                .to_array()
                .map(|channel| (channel.clamp(0.0, 1.0) * 255.0) as u8),
        )
    }
}

impl From<image::Rgb<u8>> for Color {
    fn from(rgb: image::Rgb<u8>) -> Self {
        let scale = 1.0 / 255.0;
        Self {
            value: Vec3A::from_array(rgb.0.map(|channel| channel as f32 * scale)),
        }
    }
}
