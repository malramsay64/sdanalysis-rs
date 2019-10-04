//
// voronoi.rs
// Copyright (C) 2019 Malcolm Ramsay <malramsay64@gmail.com>
// Distributed under terms of the MIT license.
//

use crate::distance::make_cartesian;
use crate::frame::Frame;
use failure::{bail, err_msg, Error};
use geo::prelude::Area;
use geo_types::{LineString, Point, Polygon};
use geos::from_geo::TryInto;

/// Compute the voronoi area for each particle in a frame
///
/// This finds the area of the voronoi polyhedron surrounding the central point of each molecule
/// within a Frame. Currently this doesn't take into account of the periodic boundary conditions.
///
pub fn voronoi_area(frame: &Frame) -> Result<Vec<f64>, Error> {
    let points: Vec<Point<f64>> = frame
        .position
        .iter()
        .map(|p| Point::new(p[0] as f64, p[1] as f64))
        .collect();

    let cell_corners: Vec<(f64, f64)> =
        [[0., 0., 0.5], [1., 0., 0.5], [1., 1., 0.5], [0., 1., 0.5]]
            .into_iter()
            .map(|p| make_cartesian(&frame.simulation_cell, p))
            .map(|p| (p[0] as f64, p[1] as f64))
            .collect();

    let geo_poly = Polygon::new(LineString::from(cell_corners), vec![]);
    let boundary: geos::Geometry = match geo_poly.try_into() {
        Ok(g) => g,
        Err(_) => bail!("Unable to create geometry"),
    };

    let polygons: Vec<_> = geos::compute_voronoi(&points, Some(&boundary), 0., false)
        .map_err(|e| err_msg(e.to_string()))?;

    Ok(polygons.into_iter().map(|p| p.area().abs()).collect())
}
