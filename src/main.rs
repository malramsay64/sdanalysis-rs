use clap::{App, Arg};
use indicatif::ProgressIterator;
use serde::Serialize;

use csv;
use gsd::GSDTrajectory;
use sdanalysis::orientational_order;

#[derive(Serialize)]
struct Row {
    molecule: usize,
    timestep: usize,
    orient_order: f64,
}

impl Row {
    fn new(molecule: usize, timestep: usize, orient_order: f64) -> Self {
        Row {
            molecule,
            timestep,
            orient_order,
        }
    }
}

fn main() {
    let matches = App::new("GSD Parser")
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

    let mut wtr = csv::Writer::from_path("test.csv").unwrap();

    for frame in GSDTrajectory::new(filename).progress() {
        for (index, value) in orientational_order(&frame, 3.5).into_iter().enumerate() {
            wtr.serialize(Row::new(index, frame.timestep as usize, value))
                .unwrap();
        }
    }
}
