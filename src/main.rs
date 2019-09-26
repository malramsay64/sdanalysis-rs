//
// main.rs
// Copyright (C) 2019 Malcolm Ramsay <malramsay64@gmail.com>
// Distributed under terms of the MIT license.
//

use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender};

use failure::Error;
use indicatif::ProgressIterator;
use serde::Serialize;
use std::sync::mpsc::channel;
use structopt::StructOpt;
use threadpool::ThreadPool;

use csv;
use gsd::GSDTrajectory;
use itertools::izip;
use sdanalysis::frame::Frame;
use sdanalysis::{num_neighbours, orientational_order};

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
    let neighbour_distance = 8.;
    let nneighs = 6;

    let n_workers = 4;
    let pool = ThreadPool::new(n_workers);

    let trj = GSDTrajectory::new(&args.filename)?;
    let num_frames = match args.num_frames {
        Some(n) => n,
        None => trj.nframes() as usize,
    };

    let progress_bar = indicatif::ProgressBar::new(num_frames as u64).with_style(
        indicatif::ProgressStyle::default_bar()
            .template("{msg}{wide_bar} {per_sec} {pos}/{len} [{elapsed_precise}/{eta_precise}]"),
    );

    let (tx, rx): (Sender<(u64, Vec<f64>)>, Receiver<(u64, Vec<f64>)>) = channel();

    let writer_thread = std::thread::spawn(move || {
        for (timestep, result) in rx.iter().take(num_frames) {
            for (index, order) in result.iter().enumerate() {
                wtr.serialize(Row::new(index, timestep as usize, *order))
                    .expect("Unable to serilize row");
            }
            progress_bar.inc(1);
        }
        wtr.flush().expect("Flushing file failed");
        progress_bar.finish();
    });

    for frame in trj.take(num_frames) {
        let tx = tx.clone();
        pool.execute(move || {
            tx.send((
                frame.timestep,
                orientational_order(&Frame::from(frame), nneighs),
            ))
            .expect("channel will be there waiting for the pool");
        });
    }

    writer_thread.join().expect("Joining threads failed");
    Ok(())
}
