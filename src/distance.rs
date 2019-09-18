//
// distance.rs
// Copyright (C) 2019 Malcolm Ramsay <malramsay64@gmail.com>
// Distributed under terms of the MIT license.
//

/// Find the minimuim image of a point
///
pub fn min_image(cell: &[f32; 6], point: &mut [f32; 3]) {
    // The cell has components [x, y, z, xy, xz, yz]
    if point[2] >= cell[2] / 2. {
        point[2] -= cell[2];
        point[1] -= cell[2] * cell[5];
        point[0] -= cell[2] * cell[4];
    } else if point[2] < cell[2] / 2. {
        point[2] += cell[2];
        point[1] += cell[2] * cell[5];
        point[0] += cell[2] * cell[4];
    }

    if point[1] >= cell[1] / 2. {
        // Only want the integer part of this number
        let i = (point[1] * (1. / cell[1]) + 0.5).trunc();

        point[1] -= i * cell[1];
        point[0] -= i * cell[1] * cell[3];
    } else if point[1] < cell[1] / 2. {
        // Only want the integer part of this number
        let i = (-point[1] * (1. / cell[1]) + 0.5).trunc();

        point[1] += i * cell[1];
        point[0] += i * cell[1] * cell[3];
    }

    if point[0] >= cell[0] / 2. {
        // Only want the integer part of this number
        let i = (point[1] * (1. / cell[1]) + 0.5).trunc();

        point[0] -= i * cell[0];
    } else if point[0] < cell[0] / 2. {
        // Only want the integer part of this number
        let i = (-point[1] * (1. / cell[1]) + 0.5).trunc();

        point[0] += i * cell[0];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}
