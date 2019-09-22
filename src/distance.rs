//
// distance.rs
// Copyright (C) 2019 Malcolm Ramsay <malramsay64@gmail.com>
// Distributed under terms of the MIT license.
//

pub fn min_image_round(cell: &[f32; 6], point: &[f32; 3]) -> [f32; 3] {
    let tilt_x = (cell[4] - cell[3] * cell[5]) * 0.5 * cell[2] + cell[3] * 0.5 * cell[1];
    let tilt_y = cell[2] * cell[5];
    let center = [
        0.5 * cell[0] + tilt_x,
        0.5 * cell[1] + tilt_y,
        0.5 * cell[2],
    ];
    let mut periodic = [
        point[0] + center[0],
        point[1] + center[1],
        point[2] + center[2],
    ];

    let z_iters = (periodic[2] / cell[2] - 0.5).ceil();
    periodic[2] -= cell[2] * z_iters;
    periodic[1] -= cell[1] * z_iters * cell[5];
    periodic[0] -= cell[0] * z_iters * cell[4];

    let y_iters = (periodic[1] / cell[1] - 0.5).ceil();
    periodic[1] -= cell[1] * y_iters;
    periodic[0] -= cell[0] * y_iters * cell[3];

    let x_iters = (periodic[0] / cell[0] - 0.5).ceil();
    periodic[0] -= cell[0] * x_iters;

    [
        periodic[0] - center[0],
        periodic[1] - center[1],
        periodic[2] - center[2],
    ]
}

/// Find the minimuim image of a point
///
pub fn min_image(cell: &[f32; 6], p: &[f32; 3]) -> [f32; 3] {
    let mut point = *p;
    // The cell has components [x, y, z, xy, xz, yz]

    // Check for wrapping in the z dimension
    if point[2] >= cell[2] / 2. {
        point[2] -= cell[2];
        point[1] -= cell[2] * cell[5];
        point[0] -= cell[2] * cell[4];
    } else if point[2] < -cell[2] / 2. {
        point[2] += cell[2];
        point[1] += cell[2] * cell[5];
        point[0] += cell[2] * cell[4];
    }

    let tilt_y = cell[2] * cell[5];
    if point[1] >= cell[1] / 2. + tilt_y {
        // Number of times for the periodicity
        let i = (point[1] * (1. / cell[1]) + 0.5).trunc();

        point[1] -= i * cell[1];
        point[0] -= i * cell[1] * cell[3];
    } else if point[1] < -cell[1] / 2. + tilt_y {
        // Number of times for the periodicity
        let i = (-point[1] * (1. / cell[1]) + 0.5).trunc();

        point[1] += i * cell[1];
        point[0] += i * cell[1] * cell[3];
    }

    let tilt_x = (cell[4] - cell[3] * cell[5]) * point[2] + cell[3] * point[1];
    if point[0] >= cell[0] / 2. + tilt_x {
        // Number of times for the periodicity
        let i = (point[0] * (1. / cell[0]) + 0.5).trunc();
        point[0] -= i * cell[0];
    } else if point[0] < -cell[0] / 2. + tilt_x {
        // Number of times for the periodicity
        let i = (-point[0] * (1. / cell[0]) + 0.5).trunc();
        point[0] += i * cell[0];
    }
    point
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_change_center() {
        let cell = [2., 2., 2., 0., 0., 0.];
        let point = [0., 0., 0.];
        assert_eq!(min_image(&cell, &point), [0., 0., 0.]);
        assert_eq!(min_image_round(&cell, &point), [0., 0., 0.]);
    }

    #[test]
    fn wrap_x_max() {
        let cell = [2., 2., 2., 0., 0., 0.];
        let point = [1., 0., 0.];
        assert_eq!(min_image(&cell, &point), [-1., 0., 0.]);
        assert_eq!(min_image_round(&cell, &point), [-1., 0., 0.]);
    }

    #[test]
    fn wrap_y_max() {
        let cell = [2., 2., 2., 0., 0., 0.];
        let point = [0., 1., 0.];
        assert_eq!(min_image(&cell, &point), [0., -1., 0.]);
        assert_eq!(min_image_round(&cell, &point), [0., -1., 0.]);
    }

    #[test]
    fn wrap_z_max() {
        let cell = [2., 2., 2., 0., 0., 0.];
        let point = [0., 0., 1.];
        assert_eq!(min_image(&cell, &point), [0., 0., -1.]);
        assert_eq!(min_image_round(&cell, &point), [0., 0., -1.]);
    }

    #[test]
    fn no_wrap_x_min() {
        let cell = [2., 2., 2., 0., 0., 0.];
        let point = [-1., 0., 0.];
        assert_eq!(min_image(&cell, &point), [-1., 0., 0.]);
        assert_eq!(min_image_round(&cell, &point), [-1., 0., 0.]);
    }

    #[test]
    fn no_wrap_y_min() {
        let cell = [2., 2., 2., 0., 0., 0.];
        let point = [0., -1., 0.];
        assert_eq!(min_image(&cell, &point), [0., -1., 0.]);
        assert_eq!(min_image_round(&cell, &point), [0., -1., 0.]);
    }

    #[test]
    fn no_wrap_z_min() {
        let cell = [2., 2., 2., 0., 0., 0.];
        let point = [0., -1., 0.];
        assert_eq!(min_image(&cell, &point), [0., -1., 0.]);
        assert_eq!(min_image_round(&cell, &point), [0., -1., 0.]);
    }

    #[test]
    fn wrap_all() {
        let cell = [2., 2., 2., 0., 0., 0.];
        let point = [1.5, 1.5, 1.5];
        assert_eq!(min_image(&cell, &point), [-0.5, -0.5, -0.5]);
        assert_eq!(min_image_round(&cell, &point), [-0.5, -0.5, -0.5]);
    }

    #[test]
    fn no_wrap_tilted() {
        let cell = [2., 2., 2., 0.5, 0., 0.];
        let point = [1.2, 0.5, 0.];
        assert_eq!(min_image(&cell, &point), [1.2, 0.5, 0.]);
        // assert_eq!(min_image_round(&cell, &point), [1.2, 0.5, 0.]);
    }
}
