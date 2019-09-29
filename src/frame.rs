//
// frame.rs
// Copyright (C) 2019 Malcolm Ramsay <malramsay64@gmail.com>
// Distributed under terms of the MIT license.
//

//! A frame type with a number of useful functions

use crate::distance::min_image;
use gsd::GSDFrame;
use nalgebra::{Quaternion, UnitQuaternion, Vector4};
use rstar::{PointDistance, RTree, RTreeObject, AABB};

pub struct Frame {
    pub timestep: u64,
    pub position: Vec<[f32; 3]>,
    pub orientation: Vec<UnitQuaternion<f32>>,
    pub image: Vec<[i32; 3]>,
    pub simulation_cell: [f32; 6],

    neighbour_tree: RTree<Position>,
}

impl From<GSDFrame> for Frame {
    fn from(frame: GSDFrame) -> Frame {
        // Preconvert the orientations to a quaternion representation
        let orientations: Vec<UnitQuaternion<f32>> = frame
            .orientation
            .into_iter()
            .map(Vector4::from)
            .map(Quaternion::from)
            .map(UnitQuaternion::from_quaternion)
            .collect();

        let neighbour_tree =
            RTree::bulk_load(array_to_points(&frame.position, &frame.simulation_cell));

        Frame {
            timestep: frame.timestep,
            position: frame.position,
            orientation: orientations,
            image: frame.image,
            simulation_cell: frame.simulation_cell,
            neighbour_tree,
        }
    }
}

impl Frame {
    pub fn neighbours_n<'a>(
        &'a self,
        n: usize,
    ) -> impl Iterator<Item = impl Iterator<Item = usize> + 'a> + 'a {
        self.position.iter().map(move |point| {
            self.neighbour_tree
                .nearest_neighbor_iter(point)
                .take(n)
                .map(|i| i.index)
        })
    }

    pub fn neighbours_cutoff<'a>(
        &'a self,
        cutoff: f32,
    ) -> impl Iterator<Item = impl Iterator<Item = usize> + 'a> + 'a {
        self.position.iter().map(move |&point| {
            self.neighbour_tree
                .locate_within_distance(point, cutoff * cutoff)
                .map(|i| i.index)
        })
    }

    pub fn len(&self) -> usize {
        self.position.len()
    }

    pub fn is_empty(&self) -> bool {
        self.position.is_empty()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Position {
    point: [f32; 3],
    index: usize,
    cell: [f32; 6],
}

impl Position {
    fn new(point: &[f32; 3], index: usize, cell: &[f32; 6]) -> Self {
        Position {
            point: *point,
            index,
            cell: *cell,
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

    fn distance_2_if_less_or_equal(&self, point: &[f32; 3], max_distance_2: f32) -> Option<f32> {
        match self.distance_2(point) {
            d if d < max_distance_2 => Some(d),
            _ => None,
        }
    }

    // I want to only compare the values exactly here
    #[allow(clippy::float_cmp)]
    fn contains_point(&self, point: &[f32; 3]) -> bool {
        self.point[0] == point[0] && self.point[1] == point[1] && self.point[2] == point[2]
    }
}

fn array_to_points(array: &[[f32; 3]], cell: &[f32; 6]) -> Vec<Position> {
    array
        // Iterate over the rows
        .iter()
        .enumerate()
        // Convert from slice to owned array
        .map(|(index, row)| Position::new(row, index, cell))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distance() {
        let test_cell = [2., 2., 2., 0., 0., 0.];
        let p = Position::new(&[0.; 3], 0, &test_cell);
        let distance = p.distance_2(&[1., 0., 0.]);
        assert_eq!(distance, 1.)
    }

    #[test]
    fn distance_periodic() {
        let test_cell = [2., 2., 2., 0., 0., 0.];
        let p = Position::new(&[0.; 3], 0, &test_cell);
        let distance = p.distance_2(&[2., 0., 0.]);
        assert_eq!(distance, 0.)
    }

    #[test]
    fn distance_within() {
        let test_cell = [1., 1., 1., 0., 0., 0.];
        let p = Position::new(&[0.; 3], 0, &test_cell);
        assert_eq!(
            p.distance_2_if_less_or_equal(&[0.5, 0., 0.], 0.5),
            Some(0.25)
        );
    }

    #[test]
    fn distance_within_periodic() {
        let test_cell = [1., 1., 1., 0., 0., 0.];
        let p = Position::new(&[0.; 3], 0, &test_cell);
        assert_eq!(p.distance_2_if_less_or_equal(&[1., 0., 0.], 0.5), Some(0.));
    }
}
