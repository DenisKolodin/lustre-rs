//! Color and pixel output

use glam::Vec3A;

pub use glam::Vec3A as Color;

pub mod colors {
    pub const WHITE: super::Color = super::Vec3A::ONE;
    pub const BLACK: super::Color = super::Vec3A::ZERO;
}

// conversion for sdr pixels
pub trait VecExt<P: image::Pixel> {
    fn to_pixel(self) -> P;
    fn from_pixel(p: P) -> Self;
}

impl VecExt<image::Rgb<u8>> for Vec3A {
    fn to_pixel(self) -> image::Rgb<u8> {
        image::Rgb::<u8>(
            self.to_array()
                .map(|channel| (channel.clamp(0.0, 1.0) * u8::MAX as f32) as u8),
        )
    }

    fn from_pixel(p: image::Rgb<u8>) -> Self {
        Self::from_array(p.0.map(|channel| (channel as f32 / u8::MAX as f32).clamp(0.0, 1.0)))
    }
}

// conversion for hdr pixels
impl VecExt<image::Rgb<f32>> for Vec3A {
    fn to_pixel(self) -> image::Rgb<f32> {
        image::Rgb::<f32>(self.to_array())
    }

    fn from_pixel(p: image::Rgb<f32>) -> Self {
        Self::from_array(p.0)
    }
}
