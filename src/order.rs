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
struct Position {
    point: [f32; 3],
    index: usize,
    cell: [f32; 6],
}

impl Position {
    fn new(point: &[f32; 3], index: usize, cell: &[f32; 6]) -> Self {
        Position {
            point: point.clone(),
            index: index,
            cell: cell.clone(),
        }
    }
}

impl RTreeObject for Position {
    type Envelope = AABB<[f32; 3]>;

    fn envelope(&self) -> Self::Envelope {
        AABB::from_point(self.point)
    }
}

impl PointDistance for Position {
    fn distance_2(&self, point: &[f32; 3]) -> f32 {
        let distance = [
            self.point[0] - point[0],
            self.point[1] - point[1],
            self.point[2] - point[2],
        ];
        let distance = min_image(&self.cell, &distance);

        distance[0] * distance[0] + distance[1] * distance[1] + distance[2] * distance[2]
    }

    fn contains_point(&self, point: &[f32; 3]) -> bool {
        self.point[0] == point[0] && self.point[1] == point[1] && self.point[2] == point[2]
    }
}

fn array_to_points(array: &Vec<[f32; 3]>, cell: &[f32; 6]) -> Vec<Position> {
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

fn neighbour_iterator<'a>(
    frame: &'a gsd::GSDFrame,
    cutoff: f32,
) -> impl Iterator<Item = Vec<Position>> + 'a {
    let points = array_to_points(&frame.position, &frame.simulation_cell);
    let tree = RTree::bulk_load(points);
    let cutoff2 = cutoff * cutoff;
    frame.position.iter().map(move |&point| {
        tree.locate_within_distance(point, cutoff2)
            .cloned()
            .collect::<Vec<Position>>()
    })
}

pub fn num_neighbours(frame: &gsd::GSDFrame, cutoff: f32) -> Vec<usize> {
    neighbour_iterator(frame, cutoff)
        .map(|neighs| neighs.len())
        .collect()
}

pub fn orientational_order(frame: &gsd::GSDFrame, cutoff: f32) -> Vec<f64> {
    let orientations: Vec<UnitQuaternion<f32>> = frame
        .orientation
        .iter()
        .map(|q| UnitQuaternion::from_quaternion(Quaternion::new(q[0], q[1], q[2], q[3])))
        .collect();

    neighbour_iterator(frame, cutoff)
        .enumerate()
        .map(|(index, neighs)| {
            neighs
                .into_iter()
                .map(|i| orientations[index].angle_to(&orientations[i.index]) as f64)
                .map(f64::cos)
                .map(|x| x * x)
                .sum::<f64>()
                / 6.
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distance() {
        let test_cell = [1., 1., 1., 0., 0., 0.];
        let p = Position::new(&[0.; 3], 0, &test_cell);
        let distance = p.distance_2(&[1., 0., 0.]);
        assert_eq!(distance, 1.)
    }

    #[test]
    fn distance_within() {
        let test_cell = [1., 1., 1., 0., 0., 0.];
        let p = Position::new(&[0.; 3], 0, &test_cell);
        assert_eq!(
            p.distance_2_if_less_or_equal(&[0.5, 0., 0.], 0.5),
            Some(0.5)
        );
    }

    #[test]
    fn distance_periodic() {
        let test_cell = [1., 1., 1., 0., 0., 0.];
        let p = Position::new(&[0.; 3], 0, &test_cell);
        assert_eq!(p.distance_2_if_less_or_equal(&[1., 0., 0.], 0.), Some(0.));
    }
}
