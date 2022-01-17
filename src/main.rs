//
// main.rs
// Copyright (C) 2019 Malcolm Ramsay <malramsay64@gmail.com>
// Distributed under terms of the MIT license.
//

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Error;
use itertools::izip;
use serde::Serialize;
use structopt::StructOpt;

use gsd::GSDTrajectory;
use trajedy::frame::Frame;
use trajedy::learning::{extract_features, run_training, Classes};
use trajedy::voronoi::voronoi_area;
use trajedy::{hexatic_order, orientational_order};

#[derive(Serialize)]
struct Row {
    molecule: usize,
    timestep: usize,
    orient_order: f32,
    hexatic_order: f32,
    class: Classes,
    area: Option<f64>,
}

struct CalcResult {
    timestep: usize,
    orient_order: Vec<f32>,
    hexatic_order: Vec<f32>,
    class: Vec<Classes>,
    area: Option<Vec<f64>>,
}

#[allow(clippy::from_over_into)]
impl Into<Vec<Row>> for CalcResult {
    fn into(self) -> Vec<Row> {
        let unwrapped_area: Box<dyn Iterator<Item = Option<f64>>> = match self.area {
            Some(a) => Box::new(a.into_iter().map(Some)),
            None => Box::new((0..).map(|_| None)),
        };
        let timestep = self.timestep as usize;
        izip!(
            0..,
            self.orient_order.into_iter(),
            self.hexatic_order.into_iter(),
            self.class.into_iter(),
            unwrapped_area,
        )
        .map(|(molecule, orient_order, hexatic_order, class, area)| Row {
            molecule,
            timestep,
            orient_order,
            hexatic_order,
            class,
            area,
        })
        .collect()
    }
}

#[derive(Debug, Clone, StructOpt)]
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

    /// Skip this many frames between configurations which are sampled
    #[structopt(long, default_value = "1")]
    skip_frames: usize,

    /// The files which are going to be used for training the machine learning model
    #[structopt(long)]
    training: Vec<String>,

    /// Whether to compute the voronoi diagram
    #[structopt(long)]
    voronoi: bool,
}

#[paw::main]
fn main(args: Args) -> Result<(), Error> {
    let nneighs = 6;
    let compute_area = args.voronoi;

    let knn = Arc::new(run_training(args.training, 100)?);

    let trj = GSDTrajectory::new(&args.filename)?;
    let num_frames = match args.num_frames {
        Some(n) => n.min(trj.nframes() as usize),
        None => trj.nframes() as usize / args.skip_frames,
    };

    let (tx, rx) = std::sync::mpsc::channel::<CalcResult>();

    let progress_bar = indicatif::ProgressBar::new(num_frames as u64).with_style(
        indicatif::ProgressStyle::default_bar()
            .template("{msg}{wide_bar} {per_sec} {pos}/{len} [{elapsed_precise}/{eta_precise}]"),
    );
    let mut wtr = csv::Writer::from_path(args.outfile)?;
    let writer_thread = std::thread::spawn(move || {
        for frame_result in rx.iter() {
            let results: Vec<Row> = frame_result.into();
            for row in results {
                wtr.serialize(row).expect("Serializing frame failed");
            }
            progress_bar.inc(1);
        }
        wtr.flush().expect("Flushing file failed");
        progress_bar.finish();
    });

    for frame in trj.step_by(args.skip_frames).take(num_frames) {
        let tx = tx.clone();
        let k = knn.clone();
        rayon::spawn_fifo(move || {
            let f = Frame::from(frame);
            let orient_order = orientational_order(&f, nneighs);
            let hexatic_order = hexatic_order(&f, nneighs);
            assert_eq!(orient_order.len(), f.len());
            let class = k
                .predict(&extract_features(&f))
                .unwrap_or_else(|_| vec![Classes::Liquid; f.len()]);
            assert_eq!(class.len(), f.len());
            let area = if compute_area {
                Some(voronoi_area(&f).unwrap())
            } else {
                None
            };
            tx.send(CalcResult {
                timestep: f.timestep as usize,
                orient_order,
                hexatic_order,
                class,
                area,
            })
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
