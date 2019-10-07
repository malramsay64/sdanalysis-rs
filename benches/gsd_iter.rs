//
// iteration.rs
// Copyright (C) 2019 Malcolm Ramsay <malramsay64@gmail.com>
// Distributed under terms of the MIT license.
//

//! Benchmarking for the GSDTrajectory iterator
//!
//! The primary purpose of this iterator is to understand how increasing the size of a step through
//! a file affects the speed of iteration.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use gsd::GSDTrajectory;
use std::path::PathBuf;

fn create_file(c: &mut Criterion) {
    let mut filename = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    filename.push("trajectory.gsd");

    let mut group = c.benchmark_group("trajectory_step_by");

    group.bench_with_input(
        BenchmarkId::from_parameter("create traj"),
        &filename,
        |b, filename| b.iter(|| GSDTrajectory::new(filename).expect("File not found")),
    );
    group.finish();
}

fn iterator_step_by(c: &mut Criterion) {
    let mut filename = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    filename.push("trajectory.gsd");

    let mut group = c.benchmark_group("trajectory_step_by");

    for steps in [1, 2, 4, 8].iter() {
        group.throughput(Throughput::Elements(1));
        group.bench_with_input(BenchmarkId::from_parameter(steps), steps, |b, &steps| {
            b.iter(|| {
                GSDTrajectory::new(&filename)
                    .expect("File not found")
                    .step_by(steps)
                    .take(2)
                    .map(|f| f.timestep)
                    .collect::<Vec<_>>()
            })
        });
    }
    group.finish();
}

criterion_group!(gsd_iter, iterator_step_by, create_file);
criterion_main!(gsd_iter);
