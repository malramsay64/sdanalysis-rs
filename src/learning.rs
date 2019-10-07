//
// learning.rs
// Copyright (C) 2019 Malcolm Ramsay <malramsay64@gmail.com>
// Distributed under terms of the MIT license.
//

use crate::frame::Frame;
use crate::knn::KNN;
use failure::Error;
use gsd::GSDTrajectory;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

pub fn extract_features(frame: &Frame) -> Vec<[f32; 6]> {
    frame
        .neighbours_n(6)
        .enumerate()
        .map(|(mol_index, neighs)| {
            let mut features = [0.; 6];
            for (i, neighbour) in neighs.enumerate() {
                features[i] = frame.orientation[mol_index].angle_to(&frame.orientation[neighbour])
            }
            features
        })
        .collect()
}

pub trait Classification: std::fmt::Debug + Clone + Copy + FromStr + PartialEq + Eq {
    fn consensus(votes: &[Self]) -> Self;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Classes {
    Liquid,
    P2,
    P2GG,
    PG,
}

impl FromStr for Classes {
    type Err = Error;

    fn from_str(s: &str) -> Result<Classes, Self::Err> {
        Ok(match s {
            s if s.contains("-p2") => Classes::P2,
            s if s.contains("-p2gg") => Classes::P2GG,
            s if s.contains("-pg") => Classes::PG,
            _ => Classes::Liquid,
        })
    }
}

impl Classification for Classes {
    fn consensus(votes: &[Self]) -> Self {
        let mut boxes = [0_usize; 4];
        for vote in votes {
            match vote {
                Self::Liquid => boxes[0] += 1,
                Self::P2 => boxes[1] += 1,
                Self::P2GG => boxes[2] += 1,
                Self::PG => boxes[3] += 1,
            }
        }
        let max_index = boxes
            .iter()
            .enumerate()
            .max_by_key(|&(_, item)| item)
            .unwrap_or((0, &0))
            .0;

        match max_index {
            0 => Self::Liquid,
            1 => Self::P2,
            2 => Self::P2GG,
            3 => Self::PG,
            _ => unreachable!("Assigning values to a class which doesn't exist"),
        }
    }
}

fn classify_file(filename: &str, index: usize) -> Result<Vec<([f32; 6], Classes)>, Error> {
    let crystal = Classes::from_str(filename)?;
    let frame: Frame = GSDTrajectory::new(&filename)?
        .get_frame(index as u64)?
        .into();
    // Initialise class to be zero for all particles
    Ok(frame
        .position
        .iter()
        .zip(extract_features(&frame))
        .filter_map(|(position, feat)| {
            match (
                position[0] / frame.simulation_cell[0],
                position[1] / frame.simulation_cell[1],
            ) {
                // The central region is crystalline
                (x, y) if x.abs() < 0.28 && y.abs() < 0.28 => Some((feat, crystal)),
                // The surrounding region is interface, so ignore
                (x, y) if x.abs() < 0.32 && y.abs() < 0.32 => None,
                _ => Some((feat, Classes::Liquid)),
            }
        })
        .collect())
}

pub fn run_training(filenames: Vec<String>, index: usize) -> Result<KNN<[f32; 6], Classes>, Error> {
    let mut knn = KNN::default();
    let (features, classes): (Vec<_>, Vec<_>) = filenames
        .iter()
        .filter_map(|f| classify_file(f, index).ok())
        .map(|i| i.into_iter())
        .flatten()
        .unzip();
    knn.fit(&features, &classes);
    Ok(knn)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}
