//
// order.rs
// Copyright (C) 2019 Malcolm Ramsay <malramsay64@gmail.com>
// Distributed under terms of the MIT license.
//

use gsd;
use nalgebra::{Quaternion, UnitQuaternion};
use rstar::{Point, RTree};
use simple_error::{bail, SimpleError, SimpleResult};
use stats::mean;
use std::convert::TryFrom;

#[derive(Clone, Copy, Debug, PartialEq)]
struct TreePoint {
    point: [f32; 3],
    index: Option<usize>,
}

impl TreePoint {
    fn new(point: &[f32; 3], index: usize) -> Self {
        TreePoint {
            point: point.clone(),
            index: Some(index),
        }
    }
}

impl TryFrom<&[f32]> for TreePoint {
    type Error = SimpleError;

    fn try_from(values: &[f32]) -> SimpleResult<Self> {
        if values.len() != 3 {
            bail!("Values doesn't have a length of 3");
        }
        let mut point = [0.; 3];
        // This panics when the slices are not the same length
        point.copy_from_slice(values);

        Ok(TreePoint { point, index: None })
    }
}

impl Point for TreePoint {
    type Scalar = f32;
    const DIMENSIONS: usize = 2;

    fn generate(generator: impl Fn(usize) -> Self::Scalar) -> Self {
        TreePoint {
            point: [generator(0), generator(1), 0.],
            index: None,
        }
    }

    fn nth(&self, index: usize) -> Self::Scalar {
        self.point[index]
    }

    fn nth_mut(&mut self, index: usize) -> &mut Self::Scalar {
        &mut self.point[index]
    }
}

fn array_to_points(array: &Vec<[f32; 3]>) -> Vec<TreePoint> {
    array
        // Iterate over the rows
        .iter()
        .enumerate()
        // Convert from slice to owned array
        .map(|(index, row)| TreePoint::new(row, index))
        .collect()
}

pub fn nearest_neighbours(positions: &Vec<[f32; 3]>, cutoff: f32) -> Vec<Vec<Option<usize>>> {
    let tree = RTree::bulk_load(array_to_points(positions));
    tree.iter()
        .map(|&i| {
            tree.locate_within_distance(i, cutoff * cutoff)
                .map(|i| i.index)
                .collect()
        })
        .collect()
}

pub fn orientational_order(frame: &gsd::GSDFrame, cutoff: f32) -> Vec<f64> {
    // Find all nearest neigbours
    let points = array_to_points(&frame.position);
    let tree = RTree::bulk_load(points.clone());
    let orientations: Vec<UnitQuaternion<f32>> = frame
        .orientation
        .iter()
        .map(|q| UnitQuaternion::from_quaternion(Quaternion::new(q[0], q[1], q[2], q[3])))
        .collect();
    // Find the relative orientation of the nearest neighbours
    points
        .iter()
        .map(|&point| {
            mean(
                tree.locate_within_distance(point, cutoff * cutoff)
                    .map(|i| {
                        orientations[point.index.unwrap()].angle_to(&orientations[i.index.unwrap()])
                    })
                    .map(f32::cos)
                    .map(|x| x * x),
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}
