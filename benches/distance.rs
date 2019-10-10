//
// distance.rs
// Copyright (C) 2019 Malcolm Ramsay <malramsay64@gmail.com>
// Distributed under terms of the MIT license.
//

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

use trajedy::distance::min_image;

fn bench_min_image(c: &mut Criterion) {
    let cell = [1., 1., 1., 0., 0., 0.];
    let point = [0.5; 3];
    c.bench_function("make_fractional", |b| {
        b.iter(|| black_box(min_image(&cell, &point)))
    });
}

fn bench_n_points(c: &mut Criterion) {
    let cell = [1., 1., 1., 0., 0., 0.];
    let mut group = c.benchmark_group("min_image_points");
    for &n in [1, 10, 100, 1000].iter() {
        group.throughput(Throughput::Elements(n));
        group.bench_with_input(
            BenchmarkId::from_parameter(n),
            &vec![[1.5; 3]; n as usize],
            |b, points| {
                b.iter(|| {
                    for p in points {
                        black_box(min_image(&cell, p));
                    }
                })
            },
        );
    }
}

fn bench_collect_n_points(c: &mut Criterion) {
    let cell = [1., 1., 1., 0., 0., 0.];
    let mut group = c.benchmark_group("min_image_points_collect");
    for &n in [1, 10, 100, 1000].iter() {
        group.throughput(Throughput::Elements(n));
        group.bench_with_input(
            BenchmarkId::from_parameter(n),
            &vec![[1.5; 3]; n as usize],
            |b, points| {
                b.iter(|| {
                    points
                        .iter()
                        .map(|p| min_image(&cell, p))
                        .collect::<Vec<_>>()
                })
            },
        );
    }
}

criterion_group!(
    benches,
    bench_min_image,
    bench_n_points,
    bench_collect_n_points
);
criterion_main!(benches);
