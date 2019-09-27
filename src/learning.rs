//
// learning.rs
// Copyright (C) 2019 Malcolm Ramsay <malramsay64@gmail.com>
// Distributed under terms of the MIT license.
//

use crate::frame::Frame;
use crate::knn::KNN;
use failure::Error;
use gsd::GSDTrajectory;

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

pub fn run_training(filenames: Vec<String>, index: usize) -> Result<KNN<[f32; 6]>, Error> {
    let mut knn = KNN::default();
    let mut features: Vec<[f32; 6]> = Vec::new();
    let mut classes: Vec<usize> = Vec::new();
    for filename in filenames {
        let crystal = 1;
        let frame: Frame = GSDTrajectory::new(&filename)?
            .get_frame(index as u64)?
            .into();
        // Initialise class to be zero for all particles
        let (mut feat_file, mut feat_class): (Vec<[f32; 6]>, Vec<usize>) = frame
            .position
            .iter()
            .zip(extract_features(&frame))
            .filter_map(|(position, feat)| {
                match (
                    position[0] / frame.simulation_cell[0],
                    position[1] / frame.simulation_cell[1],
                ) {
                    // The central region is crystalline
                    (x, y) if x.abs() < 0.3 && y.abs() < 0.3 => Some((feat, crystal)),
                    // The surrounding region is interface, so ignore
                    (x, y) if x.abs() < 0.35 && y.abs() < 0.35 => None,
                    _ => Some((feat, 0)),
                }
            })
            .unzip();
        features.append(&mut feat_file);
        classes.append(&mut feat_class);
    }
    knn.fit(&features, &classes);
    Ok(knn)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}
