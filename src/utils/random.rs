//! Miscelleanous utilities related to random number generation and random sampling
//!
//! Relies on the [rand] and [rand_distr] crates

use glam::Vec3A;
use rand::Rng;
use rand_distr::{Distribution, UnitDisc, UnitSphere};

/// Generates a random [Vec3A] within the unit sphere (radius 1).
///
/// wrapper function around [UnitSphere]'s `sample` method
pub fn rand_vec3_on_unit_sphere(rng: &mut impl Rng) -> Vec3A {
    Vec3A::from_array(UnitSphere.sample(rng))
}

#[allow(dead_code/* , reason = "Want to A/B test with this sometimes" */)]
/// Generates a random [Vec3A] within the same unit hemisphere as the given normal.
pub fn rand_vec3_on_unit_hemisphere(rng: &mut impl Rng, normal: Vec3A) -> Vec3A {
    let mut unit_v = rand_vec3_on_unit_sphere(rng);
    if unit_v.dot(normal) < 0.0 {
        unit_v = -unit_v;
    }

    unit_v
}

/// Generates a random [Vec3A] within the unit disk (radius 1).
///
/// wrapper function around [UnitDisc]'s `sample` method.
pub fn rand_vec3_in_unit_disk(rng: &mut impl Rng) -> Vec3A {
    let [x, y] = UnitDisc.sample(rng);
    Vec3A::new(x, y, 0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rand_unit_sphere() {
        let mut rng = rand::thread_rng();
        let res = rand_vec3_on_unit_sphere(&mut rng);
        assert!(
            res.is_normalized(),
            "the unit vector {res}'s length was {}",
            res.length()
        )
    }

    #[test]
    fn test_rand_unit_disk() {
        let mut rng = rand::thread_rng();
        let res = rand_vec3_in_unit_disk(&mut rng);
        assert!(
            res.length() <= 1.0,
            "expected a vector with a length <= 1, found a vector {res} with length {}",
            res.length()
        )
    }
}
