//! Various utilities
//!
//! External create wrappers, small functions, etc.

pub mod arena;
pub mod match_opts;
pub mod progress;
pub mod random;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Axis {
    X,
    Y,
    Z,
}

impl Axis {
    pub const AXES: [Axis; 3] = [Axis::X, Axis::Y, Axis::Z];
}

impl std::ops::Index<Axis> for glam::Vec3A {
    type Output = f32;

    #[inline]
    fn index(&self, index: Axis) -> &Self::Output {
        &self[index as usize]
    }
}

impl std::ops::IndexMut<Axis> for glam::Vec3A {
    #[inline]
    fn index_mut(&mut self, index: Axis) -> &mut Self::Output {
        &mut self[index as usize]
    }
}

impl From<Axis> for glam::Vec3A {
    fn from(axis: Axis) -> Self {
        glam::Vec3A::AXES[axis as usize]
    }
}
