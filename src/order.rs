//
// order.rs
// Copyright (C) 2019 Malcolm Ramsay <malramsay64@gmail.com>
// Distributed under terms of the MIT license.
//

use crate::frame::Frame;
use alga::linear::NormedSpace;
use nalgebra::{Complex, Point3, Rotation2, UnitComplex, UnitQuaternion, Vector2};
use num_traits::Zero;

pub fn num_neighbours(frame: &Frame, cutoff: f32) -> Vec<usize> {
    frame
        .neighbours_cutoff(cutoff)
        .map(|neighs| neighs.count())
        .collect()
}

/// A Helper function to comptue the orientational order
///
/// This provides a method by which to compute the orientational order. This is the component
/// which is more straitforward to test.
///
/// Returns a values in the range [0,1]
///
fn orientational_order_iter(
    reference: &UnitQuaternion<f32>,
    neighs: impl Iterator<Item = UnitQuaternion<f32>>,
    num_neighbours: usize,
) -> f32 {
    neighs.fold(0., |acc, i| acc + reference.angle_to(&i).cos().powi(2)) / num_neighbours as f32
}

/// This computes the orientational order paramter for every particle in a configuration.
///
/// The orientational order parameter, is the relative orientation of the `num_neighbours`
/// nearest particles converted into a one dimensional paramter.
///
pub fn orientational_order(frame: &Frame, num_neighbours: usize) -> Vec<f32> {
    // Calculate the orientational_order parameter for each particle
    frame
        .neighbours_n(num_neighbours)
        .enumerate()
        .map(|(index, neighs)| {
            orientational_order_iter(
                &frame.orientation[index],
                neighs.map(|n| frame.orientation[n]),
                num_neighbours,
            )
        })
        .collect()
}

/// A Helper function to comptue the hexatic order
///
/// This provides a method by which to compute the hexatic order. This is the component
/// which is more straitforward to test.
///
/// Returns a values in the range [0,1]
///
fn hexatic_order_iter(
    reference: &Point3<f32>,
    neighs: impl Iterator<Item = Point3<f32>>,
    num_neighbours: usize,
) -> f32 {
    let reference_vec = Vector2::new(0., 1.);
    neighs
        .map(|p| p - reference)
        // Calculate the rotation between two vectors
        .map(|v| Rotation2::rotation_between(&reference_vec.xy(), &v.xy()))
        // Convert the rotation to an angle, multiply by the k-fold symmetry
        .map(|c| c.angle() * num_neighbours as f32)
        // Convert the multiplied angle into a UnitComplex (rotation), then downcast to Complex
        .map(|a| UnitComplex::from_angle(a).into_inner())
        // Average all the complex numbers
        .fold(Complex::zero(), |acc, i| acc + i / num_neighbours as f32)
        .norm()
}

/// Compute the hexatic order for every particle in a configuration
///
/// The hexatic order parameter is a measure of how close the angles of the neighbouring particles
/// are to a perfect hexagon. It is calculated as
///
/// $$ \psi_k = \frac{1}{k} \sum_j^n \exp{i k \theta} $$
///
/// where $k$ is the fold of the orientational ordering.
///
pub fn hexatic_order(frame: &Frame, num_neighbours: usize) -> Vec<f32> {
    frame
        .neighbours_n(num_neighbours)
        .enumerate()
        .map(|(index, neighs)| {
            hexatic_order_iter(
                &frame.position[index],
                neighs.map(|i| frame.position[i]),
                num_neighbours,
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;
    use proptest::prelude::*;

    #[test]
    fn hexatic_order_perfect() {
        let reference = Point3::new(0., 0., 0.);
        let angles = vec![0., 60., 120., 180., 240., 300.];
        let points = angles
            .into_iter()
            .map(f32::to_radians)
            .map(f32::sin_cos)
            .map(|(x, y)| Point3::new(x, y, 0.));

        let hexatic: f32 = hexatic_order_iter(&reference, points, 6);
        assert_abs_diff_eq!(hexatic, 1.);
    }

    #[test]
    /// Ensure invariance to orientation of hexagon
    fn hexatic_order_rotated() {
        let reference = Point3::new(0., 0., 0.);
        for i in 0..360 {
            let angles = vec![0., 60., 120., 180., 240., 300.];
            let points = angles
                .into_iter()
                .map(|a| a + i as f32)
                .map(f32::to_radians)
                .map(f32::sin_cos)
                .map(|(x, y)| Point3::new(x, y, 0.));

            let hexatic: f32 = hexatic_order_iter(&reference, points, 6);
            assert_abs_diff_eq!(hexatic, 1.);
        }
    }

    proptest! {
        #[test]
        /// Ensure values well behaved [0, 1]
        fn hexatic_order_range(angles in proptest::collection::vec(0_f32..3.14, 6)) {
            let reference = Point3::new(0., 0., 0.);
            let points = angles
                .into_iter()
                .map(f32::sin_cos)
                .map(|(x, y)| Point3::new(x, y, 0.));

            let hexatic: f32 = hexatic_order_iter(&reference, points, 6);
            assert!(0. <= hexatic && hexatic <= 1.);
        }
    }

    #[test]
    fn orientational_order_perfect() {
        let reference = UnitQuaternion::from_euler_angles(0., 0., 0.);
        let angles = vec![0.; 6];
        let points = angles
            .into_iter()
            .map(|a| UnitQuaternion::from_euler_angles(0., 0., a));

        let orient_order: f32 = orientational_order_iter(&reference, points, 6);
        assert_abs_diff_eq!(orient_order, 1.);
    }

    #[test]
    /// Ensure invariance to orientation of hexagon
    fn orientational_order_rotated() {
        for i in 0..360 {
            let reference = UnitQuaternion::from_euler_angles(0., 0., (i as f32).to_radians());
            let angles = vec![(i as f32).to_radians(); 6];
            let points = angles
                .into_iter()
                .map(|a| UnitQuaternion::from_euler_angles(0., 0., a));

            let orient_order: f32 = orientational_order_iter(&reference, points, 6);
            assert_abs_diff_eq!(orient_order, 1.);
        }
    }

    proptest! {
        #[test]
        /// Ensure values well behaved [0, 1]
        fn orientational_order_range(angles in proptest::collection::vec(0_f32..3.14, 6)) {
            let reference = UnitQuaternion::from_euler_angles(0., 0., 0.);
            let points = angles
                .into_iter()
                .map(|a| UnitQuaternion::from_euler_angles(0., 0., a));

            let orient_order: f32 = orientational_order_iter(&reference, points, 6);
            assert!(0. <= orient_order);
            assert!(orient_order <= 1.);
        }
    }
}
