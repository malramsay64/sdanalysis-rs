//
// order.rs
// Copyright (C) 2019 Malcolm Ramsay <malramsay64@gmail.com>
// Distributed under terms of the MIT license.
//

use crate::frame::Frame;
use adjacent_iterator::CyclicAdjacentPairIterator;
use alga::linear::NormedSpace;
use nalgebra::{Complex, Point3, UnitComplex};
use num_traits::Zero;

pub fn num_neighbours(frame: &Frame, cutoff: f32) -> Vec<usize> {
    frame
        .neighbours_cutoff(cutoff)
        .map(|neighs| neighs.count())
        .collect()
}

/// This computes the orientational order paramter for every particle in a configuration.
///
/// The orientational order parameter, is the relative orientation of the `num_neighbours`
/// nearest particles converted into a one dimensional paramter.
///
pub fn orientational_order(frame: &Frame, num_neighbours: usize) -> Vec<f64> {
    // Calculate the orientational_order parameter for each particle
    frame
        .neighbours_n(num_neighbours)
        .enumerate()
        .map(|(index, neighs)| {
            neighs
                .map(|i| {
                    f64::cos(f64::from(
                        frame.orientation[index].angle_to(&frame.orientation[i]),
                    ))
                })
                .map(|i| i * i)
                // Take the mean using an online algorithm
                .collect::<stats::OnlineStats>()
                .mean()
        })
        .collect()
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
pub fn hexatic_order(frame: &Frame, num_neighbours: usize) -> Vec<f64> {
    frame
        .neighbours_n(num_neighbours)
        .enumerate()
        .map(|(index, neighs)| {
            let center = Point3::from(frame.position[index]);
            let hexatic: Complex<f32> = neighs
                .map(|i| Point3::from(frame.position[i]))
                .map(|p| p - center)
                .cyclic_adjacent_pairs()
                .map(|(v1, v2)| UnitComplex::rotation_between(&v1.xy(), &v2.xy()).into_inner())
                .fold(Complex::zero(), |acc, i| acc + i);
            f64::from(hexatic.norm())
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
}
