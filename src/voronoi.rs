//
// voronoi.rs
// Copyright (C) 2019 Malcolm Ramsay <malramsay64@gmail.com>
// Distributed under terms of the MIT license.
//

use crate::distance::make_cartesian;
use crate::distance::min_image;
use crate::frame::Frame;
use anyhow::Error;
use std::convert::TryFrom;
use voronoi::{make_polygons, voronoi, Cell, Point};

/// Compute the voronoi area for each particle in a frame
///
/// This finds the area of the voronoi polyhedron surrounding the central point of each molecule
/// within a Frame. Currently this doesn't take into account of the periodic boundary conditions.
///
pub fn voronoi_area(frame: &Frame) -> Result<Vec<f64>, Error> {
    let points: Vec<Point> = frame
        .position
        .iter()
        // Hoomd allows positions to be outside the cell, so this wraps all the points to be inside
        // the simulation cell.
        .map(|p| min_image(&frame.simulation_cell, p))
        .map(|p| Point::new(f64::from(p[0]), f64::from(p[1])))
        .collect();

    let cell_corners: Vec<_> = [[0., 0., 0.5], [1., 0., 0.5], [1., 1., 0.5], [0., 1., 0.5]]
        .iter()
        .map(|p| make_cartesian(&frame.simulation_cell, p))
        .map(|p| Point::new(f64::from(p[0]), f64::from(p[1])))
        .collect();

    let boundary: Cell = Cell::try_from(cell_corners)?;

    let polygons: Vec<_> = make_polygons(&voronoi(points, &boundary));

    Ok(polygons.into_iter().map(shoelace).collect())
}

fn shoelace(polygon: Vec<Point>) -> f64 {
    polygon
        .iter()
        .zip(polygon.iter().cycle().skip(1))
        .map(|(curr, next)| (next.x() + curr.x()) * (next.y() - curr.y()))
        .sum::<f64>()
        .abs()
        / 2.
}

#[cfg(test)]
mod tests {
    use super::*;
    use voronoi::Point;

    #[test]
    fn simple_area() {
        let points = vec![Point::new(0., 1.), Point::new(2., 3.), Point::new(4., 7.)];
        assert_eq!(shoelace(points), 2.)
    }
}
