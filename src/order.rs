//
// order.rs
// Copyright (C) 2019 Malcolm Ramsay <malramsay64@gmail.com>
// Distributed under terms of the MIT license.
//

use crate::frame::Frame;
use gsd;
use nalgebra::{Quaternion, UnitQuaternion};

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
    // Preconvert the orientations to a quaternion representation
    let orientations: Vec<UnitQuaternion<f32>> = frame
        .orientation
        .iter()
        .map(|q| UnitQuaternion::from_quaternion(Quaternion::new(q[0], q[1], q[2], q[3])))
        .collect();

    // Calculate the orientational_order parameter for each particle
    frame
        .neighbours_n(num_neighbours)
        .enumerate()
        .map(|(index, neighs)| {
            neighs
                .map(|i| f64::from(orientations[index].angle_to(&orientations[i])))
                .map(f64::cos)
                .map(|x| x * x)
                // Take the mean using an online algorithm
                .collect::<stats::OnlineStats>()
                .mean()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

}
