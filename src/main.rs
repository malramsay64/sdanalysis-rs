//
// main.rs
// Copyright (C) 2019 Malcolm Ramsay <malramsay64@gmail.com>
// Distributed under terms of the MIT license.
//

use std::path::PathBuf;

use failure::Error;
use indicatif::ProgressIterator;
use serde::Serialize;
use structopt::StructOpt;

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

#[derive(Debug, StructOpt)]
struct Args {
    /// The gsd file to process
    #[structopt()]
    filename: String,

    /// File to save csv data to
    #[structopt(parse(from_os_str))]
    outfile: PathBuf,

    /// The number of frames to read
    #[structopt(short, long)]
    num_frames: Option<usize>,
}

#[paw::main]
fn main(args: Args) -> Result<(), Error> {
    let mut wtr = csv::Writer::from_path(args.outfile)?;
    let neighbour_distance = 3.5;

    let trj = GSDTrajectory::new(&args.filename)?;
    let num_frames = match args.num_frames {
        Some(n) => n,
        None => trj.nframes() as usize,
    };

    for frame in trj.take(num_frames).progress() {
        for (index, order, neighs) in izip!(
            0..,
            orientational_order(&frame, neighbour_distance),
            num_neighbours(&frame, neighbour_distance),
        ) {
            wtr.serialize(Row::new(index, frame.timestep as usize, order, neighs))?;
        }
    }
    wtr.flush().expect("Flushing file failed");
    Ok(())
}
