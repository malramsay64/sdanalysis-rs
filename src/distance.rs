//
// distance.rs
// Copyright (C) 2019 Malcolm Ramsay <malramsay64@gmail.com>
// Distributed under terms of the MIT license.
//

#[inline]
fn make_fractional(cell: &[f32; 6], point: &[f32; 3]) -> [f32; 3] {
    let mut p = [0.; 3];

    p[0] = point[0] + 0.5 * cell[0];
    p[1] = point[1] + 0.5 * cell[1];
    p[2] = point[2] + 0.5 * cell[2];

    p[0] -= (cell[4] - cell[5] * cell[3]) * point[2] + cell[3] * point[1];
    p[1] -= cell[5] * point[2];

    p[0] /= cell[0];
    p[1] /= cell[1];
    p[2] /= cell[2];

    p
}

#[inline]
pub(crate) fn make_cartesian(cell: &[f32; 6], point: &[f32; 3]) -> [f32; 3] {
    let mut p = [0.; 3];

    p[0] = (point[0] - 0.5) * cell[0];
    p[1] = (point[1] - 0.5) * cell[1];
    p[2] = (point[2] - 0.5) * cell[2];

    p[0] += cell[3] * p[1] + cell[4] * p[2];
    p[1] += cell[5] * p[2];

    p
}

#[inline]
pub fn min_image(cell: &[f32; 6], point: &[f32; 3]) -> [f32; 3] {
    let mut fractional = make_fractional(cell, point);
    fractional[0] -= fractional[0].floor();
    fractional[1] -= fractional[1].floor();
    fractional[2] -= fractional[2].floor();
    make_cartesian(cell, &fractional)
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::{assert_abs_diff_eq, assert_relative_eq};
    use proptest::prelude::*;

    #[test]
    fn no_change_center() {
        let cell = [2., 2., 2., 0., 0., 0.];
        let point = [0., 0., 0.];
        assert_eq!(min_image(&cell, &point), [0., 0., 0.]);
    }

    #[test]
    fn wrap_x_max() {
        let cell = [2., 2., 2., 0., 0., 0.];
        let point = [1., 0., 0.];
        assert_eq!(min_image(&cell, &point), [-1., 0., 0.]);
    }

    #[test]
    fn wrap_y_max() {
        let cell = [2., 2., 2., 0., 0., 0.];
        let point = [0., 1., 0.];
        assert_eq!(min_image(&cell, &point), [0., -1., 0.]);
    }

    #[test]
    fn wrap_z_max() {
        let cell = [2., 2., 2., 0., 0., 0.];
        let point = [0., 0., 1.];
        assert_eq!(min_image(&cell, &point), [0., 0., -1.]);
    }

    #[test]
    fn no_wrap_x_min() {
        let cell = [2., 2., 2., 0., 0., 0.];
        let point = [-1., 0., 0.];
        assert_eq!(min_image(&cell, &point), [-1., 0., 0.]);
    }

    #[test]
    fn no_wrap_y_min() {
        let cell = [2., 2., 2., 0., 0., 0.];
        let point = [0., -1., 0.];
        assert_eq!(min_image(&cell, &point), [0., -1., 0.]);
    }

    #[test]
    fn no_wrap_z_min() {
        let cell = [2., 2., 2., 0., 0., 0.];
        let point = [0., -1., 0.];
        assert_eq!(min_image(&cell, &point), [0., -1., 0.]);
    }

    #[test]
    fn wrap_all() {
        let cell = [2., 2., 2., 0., 0., 0.];
        let point = [1.5, 1.5, 1.5];
        assert_eq!(min_image(&cell, &point), [-0.5, -0.5, -0.5]);
    }

    #[test]
    fn no_wrap_tilted() {
        let cell = [2., 2., 2., 0.5, 0., 0.];
        let point = [1.2, 0.5, 0.];
        assert_eq!(min_image(&cell, &point), [1.2, 0.5, 0.]);
    }

    proptest! {
        #[test]
        fn make_cartesian_large(x in 0_f32..1_f32, y in 0_f32..1_f32, z in 0_f32..1_f32) {
            let cell = [2., 2., 2., 0., 0., 0.];
            let point = make_cartesian(&cell, &[x, y, z]);
            assert_eq!(point, [-1. + 2.* x, -1. + 2.*y, -1. + 2. * z]);
        }
    }

    #[test]
    fn make_fractional_large() {
        let cell = [2., 2., 2., 0., 0., 0.];
        let point = make_fractional(&cell, &[1., 1., 1.]);
        assert_eq!(point, [1., 1., 1.]);
    }

    proptest! {
        #[test]
        fn within_cell(x in 0_f32..1_f32, y in 0_f32..1_f32, z in 0_f32..1_f32) {
            let cell = [1., 1., 1., 0., 0., 0.];
            let point = make_cartesian(&cell, &[x, y, z]);
            assert_eq!(min_image(&cell, &point), point);
        }
    }

    #[test]
    fn to_cartesian_xy() {
        let cell = [1., 1., 1., 0.5, 0., 0.];
        let point = make_cartesian(&cell, &[1., 1., 1.]);
        assert_eq!(point, [0.75, 0.5, 0.5]);
    }

    #[test]
    fn to_cartesian_xz() {
        let cell = [1., 1., 1., 0., 0.5, 0.];
        let point = make_cartesian(&cell, &[1., 1., 1.]);
        assert_eq!(point, [0.75, 0.5, 0.5]);
    }

    #[test]
    fn to_cartesian_yz() {
        let cell = [1., 1., 1., 0., 0., 0.5];
        let point = make_cartesian(&cell, &[1., 1., 1.]);
        assert_eq!(point, [0.5, 0.75, 0.5]);
    }

    #[test]
    fn to_cartesian_xy_xz() {
        let cell = [1., 1., 1., 0.5, 0.5, 0.];
        let point = make_cartesian(&cell, &[1., 1., 1.]);
        assert_eq!(point, [1.0, 0.5, 0.5]);
    }

    #[test]
    fn to_cartesian_xy_yz() {
        let cell = [1., 1., 1., 0.5, 0., 0.5];
        let point = make_cartesian(&cell, &[1., 1., 1.]);
        assert_eq!(point, [0.75, 0.75, 0.5]);
    }

    #[test]
    fn to_cartesian_xz_yz() {
        let cell = [1., 1., 1., 0., 0.5, 0.5];
        let point = make_cartesian(&cell, &[1., 1., 1.]);
        assert_eq!(point, [0.75, 0.75, 0.5]);
    }

    #[test]
    fn to_cartesian_xy_xz_yz() {
        let cell = [1., 1., 1., 0.5, 0.5, 0.5];
        let point = make_cartesian(&cell, &[1., 1., 1.]);
        assert_eq!(point, [1.0, 0.75, 0.5]);
    }

    proptest! {
        #[test]
        fn to_cartesian(x in 0_f32..1_f32, y in 0_f32..1_f32, z in 0_f32..1_f32) {
            prop_assume!(x > 0. && y > 0. && z > 0.);
            let cell = [1., 1., 1., 0., 0., 0.];
            let point = [x, y, z];
            assert_eq!(make_cartesian(&cell, &point), [x-0.5, y-0.5, z-0.5]);
        }
    }

    proptest! {
        #[test]
        fn to_fractional(x in 0_f32..1_f32, y in 0_f32..1_f32, z in 0_f32..1_f32) {
            prop_assume!(x > 0. && y > 0. && z > 0.);
            let cell = [1., 1., 1., 0., 0., 0.];
            let point = [x-0.5, y-0.5, z-0.5];
            assert_eq!(make_fractional(&cell, &point), [x, y, z]);
        }
    }

    proptest! {
        #[test]
        fn roundtrip_position(x in 0_f32..1_f32, y in 0_f32..1_f32, z in 0_f32..1_f32) {
            prop_assume!(x > 0. && y > 0. && z > 0.);
            let cell = [1., 1., 1., 0., 0., 0.];
            let point = [x, y, z];
            let roundtrip = make_fractional(&cell, &make_cartesian(&cell, &point));
            assert_eq!(roundtrip, point);
        }
    }

    proptest! {
        #[test]
        fn roundtrip_position_large(x in 0_f32..1_f32, y in 0_f32..1_f32, z in 0_f32..1_f32) {
            prop_assume!(x > 0. && y > 0. && z > 0.);
            let cell = [2., 2., 2., 0., 0., 0.];
            let point = [x, y, z];
            let roundtrip = make_fractional(&cell, &make_cartesian(&cell, &point));
            assert_eq!(roundtrip, point);
        }
    }

    proptest! {
        #[test]
        fn roundtrip_position_abnormal(x in 0_f32..1_f32, y in 0_f32..1_f32, z in 0_f32..1_f32) {
            prop_assume!(x > 0. && y > 0. && z > 0.);
            let cell = [2., 3., 4., 0., 0., 0.];
            let point = [x, y, z];
            let roundtrip = make_fractional(&cell, &make_cartesian(&cell, &point));
            assert_abs_diff_eq!(roundtrip[0], point[0]);
            assert_abs_diff_eq!(roundtrip[1], point[1]);
            assert_abs_diff_eq!(roundtrip[2], point[2]);
        }
    }

    proptest! {
        #[test]
        fn roundtrip_tilted(
            x in 0_f32..1_f32,
            y in 0_f32..1_f32,
            z in 0_f32..1_f32,
            xy in -1_f32..1_f32,
            xz in -1_f32..1_f32,
            yz in -1_f32..1_f32
        ) {
            prop_assume!(x > 0. && y > 0. && z > 0.);
            let cell = [1., 1., 1., xy, xz, yz];
            let point = [x, y, z];

            println!("Cartesian: {:?}", make_cartesian(&cell, &point));
            let roundtrip = make_fractional(&cell, &make_cartesian(&cell, &point));
            assert_abs_diff_eq!(roundtrip[0], point[0]);
            assert_abs_diff_eq!(roundtrip[1], point[1]);
            assert_abs_diff_eq!(roundtrip[2], point[2]);
        }
    }

    proptest! {
        #[test]
        fn roundtrip_tilted_outside(
            x in -1_f32..2_f32,
            y in -1_f32..2_f32,
            z in -1_f32..2_f32,
            xy in -1_f32..1_f32,
            xz in -1_f32..1_f32,
            yz in -1_f32..1_f32
        ) {
            let cell = [1., 1., 1., xy, xz, yz];
            let point = [x, y, z];

            let roundtrip = make_fractional(&cell, &make_cartesian(&cell, &point));
            assert_relative_eq!(roundtrip[0], point[0], epsilon = 4.*std::f32::EPSILON);
            assert_relative_eq!(roundtrip[1], point[1], epsilon = 4.*std::f32::EPSILON);
            assert_relative_eq!(roundtrip[2], point[2], epsilon = 4.*std::f32::EPSILON);
        }
    }

    proptest! {
        #[test]
        fn within_cell_tilted(
            x in 0_f32..1_f32,
            y in 0_f32..1_f32,
            z in 0_f32..1_f32,
            xy in -1_f32..1_f32,
            xz in -1_f32..1_f32,
            yz in -1_f32..1_f32
        ) {
            prop_assume!(x > 0. && y > 0. && z > 0.);
            let cell = [1., 1., 1., xy, xz, yz];
            let point = make_cartesian(&cell, &[x, y, z]);
            let min_image_point = min_image(&cell, &point);
            assert_abs_diff_eq!(min_image_point[0], point[0]);
            assert_abs_diff_eq!(min_image_point[1], point[1]);
            assert_abs_diff_eq!(min_image_point[2], point[2]);
        }
    }

    proptest! {
        #[test]
        fn outside_cell_tilted(
            x in -1_f32..2_f32,
            y in -1_f32..2_f32,
            z in -1_f32..2_f32,
            xy in -1_f32..1_f32,
            xz in -1_f32..1_f32,
            yz in -1_f32..1_f32
        ) {
            let cell = [1., 1., 1., xy, xz, yz];
            let point = make_cartesian(&cell, &[x, y, z]);
            let min_image_point = min_image(&cell, &point);
            let point_frac = make_fractional(&cell, &min_image_point);
            println!("Point: {:?}\nMin P: {:?}\nFract: {:?}", point, min_image_point, point_frac);

            assert!(point_frac[0] <= 1.);
            assert!(point_frac[0] > -1.*std::f32::EPSILON);
            assert!(point_frac[1] <= 1.);
            assert!(point_frac[0] > -1.*std::f32::EPSILON);
            assert!(point_frac[2] <= 1.);
            assert!(point_frac[0] > -1.*std::f32::EPSILON);
        }
    }
}
