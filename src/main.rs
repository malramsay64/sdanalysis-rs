use clap::{App, Arg};
use indicatif::ProgressIterator;
use serde::Serialize;

use csv;
use gsd::GSDTrajectory;
use itertools::izip;
use sdanalysis::{num_neighbours, orientational_order};

#[derive(Serialize)]
struct Row {
    molecule: usize,
    timestep: usize,
    orient_order: f64,
    num_neighbours: usize,
}

impl Row {
    fn new(molecule: usize, timestep: usize, orient_order: f64, num_neighbours: usize) -> Self {
        Row {
            molecule,
            timestep,
            orient_order,
            num_neighbours,
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = App::new("Orientational Analysis")
        .version("0.1")
        .author("Malcolm Ramsay")
        .arg(
            Arg::with_name("filename")
                .index(1)
                .required(true)
                .help("File to process"),
        )
        .arg(
            Arg::with_name("outfile")
                .index(2)
                .required(true)
                .help("File to output csv data to"),
        )
        .get_matches();

    let filename = matches.value_of("filename").unwrap();
    let outfile = matches.value_of("outfile").unwrap();

    let mut wtr = csv::Writer::from_path(outfile)?;
    let neighbour_distance = 3.5;

    for frame in GSDTrajectory::new(filename)?.take(100).progress() {
        for (index, order, neighs) in izip!(
            0..,
            orientational_order(&frame, neighbour_distance),
            num_neighbours(&frame, neighbour_distance),
        ) {
            wtr.serialize(Row::new(index, frame.timestep as usize, order, neighs))
                .unwrap();
        }
    }
    wtr.flush().expect("Flushing file failed");
    Ok(())
}
