//
// order.rs
// Copyright (C) 2019 Malcolm Ramsay <malramsay64@gmail.com>
// Distributed under terms of the MIT license.
//

use crate::frame::Frame;

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

#[cfg(test)]
mod tests {
    use super::*;
}
