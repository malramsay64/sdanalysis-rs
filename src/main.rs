//
// main.rs
// Copyright (C) 2019 Malcolm Ramsay <malramsay64@gmail.com>
// Distributed under terms of the MIT license.
//

use std::path::PathBuf;
use std::sync::Arc;

use failure::Error;
use itertools::izip;
use serde::Serialize;
use structopt::StructOpt;
use threadpool::ThreadPool;

use csv;
use gsd::GSDTrajectory;
use sdanalysis::frame::Frame;
use sdanalysis::learning::{extract_features, run_training};
use sdanalysis::orientational_order;

#[derive(Serialize)]
struct Row {
    molecule: usize,
    timestep: usize,
    orient_order: f64,
    class: usize,
}

#[derive(Debug, StructOpt)]
struct Args {
    /// The gsd file to process
    #[structopt()]
    filename: String,

    /// File to save csv data to
    #[structopt(parse(from_os_str))]
    outfile: PathBuf,

    /// The number of frames to read. By default this is the total number of frames in the
    /// trajecotry. Where a number larger than the total number of frames in the trajectory is
    /// specified, we use the number of frames in the trajectory.
    #[structopt(short, long)]
    num_frames: Option<usize>,

    /// The files which are going to be used for training the machine learning model
    #[structopt(long)]
    training: Vec<String>,
}

#[paw::main]
fn main(args: Args) -> Result<(), Error> {
    let nneighs = 6;
    let n_workers = 4;

    let knn = Arc::new(run_training(args.training, 100)?);

    let trj = GSDTrajectory::new(&args.filename)?;
    let num_frames = match args.num_frames {
        Some(n) => n.min(trj.nframes() as usize),
        None => trj.nframes() as usize,
    };

    let (tx, rx) = std::sync::mpsc::channel::<(u64, Vec<f64>, Vec<usize>)>();

    let progress_bar = indicatif::ProgressBar::new(num_frames as u64).with_style(
        indicatif::ProgressStyle::default_bar()
            .template("{msg}{wide_bar} {per_sec} {pos}/{len} [{elapsed_precise}/{eta_precise}]"),
    );
    let mut wtr = csv::Writer::from_path(args.outfile)?;
    let writer_thread = std::thread::spawn(move || {
        for (timestep, result, classification) in rx.iter() {
            for (index, &order, &class) in izip!(0.., result.iter(), classification.iter()) {
                wtr.serialize(Row {
                    molecule: index,
                    timestep: timestep as usize,
                    orient_order: order,
                    class,
                })
                .expect("Unable to serilize row");
            }
            progress_bar.inc(1);
        }
        wtr.flush().expect("Flushing file failed");
        progress_bar.finish();
    });

    let pool = ThreadPool::new(n_workers);
    for frame in trj.take(num_frames) {
        let tx = tx.clone();
        let k = knn.clone();
        pool.execute(move || {
            let f = Frame::from(frame);
            tx.send((
                f.timestep,
                orientational_order(&f, nneighs),
                k.clone()
                    .predict(&extract_features(&f))
                    .unwrap_or(vec![0; f.len()]),
            ))
            .expect("channel will be there waiting for the pool");
        });
    }

    // There is a clone of tx for each frame in the trajectory, each of which have called send.
    // However, that still leaves the initial copy, so here the initial transmitter is dropped
    // which means the writer thread will no longer be waiting for a final value to be sent.
    drop(tx);

    writer_thread.join().expect("Joining threads failed");
    Ok(())
}
