//
// analysis.rs
// Copyright (C) 2019 Malcolm Ramsay <malramsay64@gmail.com>
// Distributed under terms of the MIT license.
//

//! Benchmark the different analysis routines I am using
//!
//! This checks to see which parts of the analysis are slow, as a place where I can focus my
//! efforts in optimisation.
//!

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use failure::Error;
use gsd::GSDTrajectory;
use trajedy::frame::Frame;
use trajedy::learning::{extract_features, run_training};
use trajedy::orientational_order;
use trajedy::voronoi::voronoi_area;

const TEST_FILE: &str = "trajectory.gsd";

fn bench_create_frame(c: &mut Criterion) -> Result<(), Error> {
    let frame = GSDTrajectory::new(TEST_FILE)?.get_frame(1)?;
    c.bench_with_input(
        BenchmarkId::new("create_frame", TEST_FILE),
        &frame,
        |b, f| b.iter(|| Frame::from(f.clone())),
    );
    Ok(())
}

fn bench_get_timestep(c: &mut Criterion) -> Result<(), Error> {
    let frame: Frame = GSDTrajectory::new(TEST_FILE)?.get_frame(1)?.into();
    c.bench_function("frame_op", |b| b.iter(|| frame.timestep));
    Ok(())
}

fn bench_order(c: &mut Criterion) -> Result<(), Error> {
    let frame: Frame = GSDTrajectory::new(TEST_FILE)?.get_frame(1)?.into();
    c.bench_with_input(
        BenchmarkId::new("orientational_order", TEST_FILE),
        &frame,
        |b, f| b.iter(|| orientational_order(f, 6)),
    );
    Ok(())
}

fn bench_features(c: &mut Criterion) -> Result<(), Error> {
    let frame: Frame = GSDTrajectory::new(TEST_FILE)?.get_frame(1)?.into();
    c.bench_with_input(
        BenchmarkId::new("feature_creation", TEST_FILE),
        &frame,
        |b, f| b.iter(|| extract_features(f)),
    );
    Ok(())
}

fn bench_predict(c: &mut Criterion) -> Result<(), Error> {
    let frame: Frame = GSDTrajectory::new(TEST_FILE)?.get_frame(1)?.into();
    let knn = run_training(vec![String::from(TEST_FILE)], 2)?;
    let features = extract_features(&frame);
    c.bench_with_input(
        BenchmarkId::new("knn_prediction", TEST_FILE),
        &features,
        |b, feat| b.iter(|| knn.predict(feat)),
    );
    Ok(())
}

fn bench_voronoi(c: &mut Criterion) -> Result<(), Error> {
    let frame: Frame = GSDTrajectory::new(TEST_FILE)?.get_frame(1)?.into();
    c.bench_with_input(BenchmarkId::new("voronoi", TEST_FILE), &frame, |b, f| {
        b.iter(|| voronoi_area(f))
    });
    Ok(())
}

criterion_group! {
    name = creation;
    config = Criterion::default().sample_size(10);
    targets = bench_create_frame
}
criterion_group! {
    name = frame_ops;
    config = Criterion::default();
    targets = bench_get_timestep
}
criterion_group! {
    name = analysis;
    config = Criterion::default().sample_size(10);
    targets = bench_order, bench_features, bench_predict, bench_voronoi
}
criterion_main!(analysis);
