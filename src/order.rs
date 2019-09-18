//
// order.rs
// Copyright (C) 2019 Malcolm Ramsay <malramsay64@gmail.com>
// Distributed under terms of the MIT license.
//

use crate::distance::min_image;
use gsd;
use nalgebra::{Quaternion, UnitQuaternion};
use rstar::{PointDistance, RTree, RTreeObject, AABB};
use stats::mean;

#[derive(Clone, Copy, Debug, PartialEq)]
struct Position<'a> {
    point: [f32; 3],
    index: usize,
    cell: &'a [f32; 6],
}

impl<'a> Position<'a> {
    fn new(point: &[f32; 3], index: usize, cell: &'a [f32; 6]) -> Self {
        Position {
            point: point.clone(),
            index: index,
            cell,
        }
    }
}

impl RTreeObject for Position<'_> {
    type Envelope = AABB<[f32; 3]>;

    fn envelope(&self) -> Self::Envelope {
        AABB::from_point(self.point)
    }
}

impl PointDistance for Position<'_> {
    fn distance_2(&self, point: &[f32; 3]) -> f32 {
        let mut distance = [
            self.point[0] - point[0],
            self.point[1] - point[0],
            self.point[2] - point[2],
        ];
        min_image(self.cell, &mut distance);

        distance[0] * distance[0] + distance[1] * distance[1] + distance[2] * distance[2]
    }

    fn contains_point(&self, point: &[f32; 3]) -> bool {
        self.point[0] == point[0] && self.point[1] == point[1] && self.point[2] == point[2]
    }
}

fn array_to_points<'a>(array: &Vec<[f32; 3]>, cell: &'a [f32; 6]) -> Vec<Position<'a>> {
    array
        // Iterate over the rows
        .iter()
        .enumerate()
        // Convert from slice to owned array
        .map(|(index, row)| Position::new(row, index, cell))
        .collect()
}

pub fn nearest_neighbours(
    positions: &Vec<[f32; 3]>,
    cutoff: f32,
    cell: &[f32; 6],
) -> Vec<Vec<usize>> {
    let tree = RTree::bulk_load(array_to_points(positions, cell));
    positions
        .iter()
        .map(|&i| {
            tree.locate_within_distance(i, cutoff * cutoff)
                .map(|i| i.index)
                .collect()
        })
        .collect()
}

pub fn orientational_order(frame: &gsd::GSDFrame, cutoff: f32) -> Vec<f64> {
    // Find all nearest neigbours
    let points = array_to_points(&frame.position, &frame.simulation_cell);
    let tree = RTree::bulk_load(points);
    let orientations: Vec<UnitQuaternion<f32>> = frame
        .orientation
        .iter()
        .map(|q| UnitQuaternion::from_quaternion(Quaternion::new(q[0], q[1], q[2], q[3])))
        .collect();
    // Find the relative orientation of the nearest neighbours
    frame
        .position
        .iter()
        .enumerate()
        .map(|(index, &point)| {
            mean(
                tree.locate_within_distance(point, cutoff * cutoff)
                    .map(|i| orientations[index].angle_to(&orientations[i.index]) as f64)
                    .map(f64::cos)
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
