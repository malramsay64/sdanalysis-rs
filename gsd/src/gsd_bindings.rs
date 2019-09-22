//
// gsd_bindings.rs
// Copyright (C) 2019 Malcolm Ramsay <malramsay64@gmail.com>
// Distributed under terms of the MIT license.
//

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub type GSDHandle = gsd_handle;
pub type GSDIndexEntry = gsd_index_entry;

use failure::{err_msg, Error};
use std::convert::TryInto;

enum GSDType {
    UINT8,
    UINT16,
    UINT32,
    UINT64,
    INT8,
    INT16,
    INT32,
    INT64,
    FLOAT,
    DOUBLE,
}

impl GSDType {
    pub fn new<T: TryInto<usize>>(c_id: T) -> Result<GSDType, Error> {
        match c_id.try_into().unwrap_or(0) {
            0 => Err(err_msg("The type 0 is an error type")),
            1 => Ok(GSDType::UINT8),
            2 => Ok(GSDType::UINT16),
            3 => Ok(GSDType::UINT32),
            4 => Ok(GSDType::UINT64),
            5 => Ok(GSDType::INT8),
            6 => Ok(GSDType::INT16),
            7 => Ok(GSDType::INT32),
            8 => Ok(GSDType::INT64),
            9 => Ok(GSDType::FLOAT),
            10 => Ok(GSDType::DOUBLE),
            _ => Err(err_msg("The type index doens't exist")),
        }
    }

    pub fn size(&self) -> usize {
        match self {
            GSDType::UINT8 => 1,
            GSDType::UINT16 => 2,
            GSDType::UINT32 => 4,
            GSDType::UINT64 => 8,
            GSDType::INT8 => 1,
            GSDType::INT16 => 2,
            GSDType::INT32 => 4,
            GSDType::INT64 => 8,
            GSDType::FLOAT => 4,
            GSDType::DOUBLE => 8,
        }
    }
}

impl GSDIndexEntry {
    pub fn type_size(&self) -> Result<usize, Error> {
        GSDType::new(self.type_).map(|s| s.size())
    }

    pub fn expected_size(&self) -> Result<usize, Error> {
        self.type_size()
            .map(|s| s * self.N as usize * self.M as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    /// Ensure the size of the Rust types match those defined in the C implementaion
    fn gsd_type_size() {
        for i in 0..20 {
            let rust_ver = GSDType::new(i).map(|s| s.size()).unwrap_or(0);
            let c_ver = unsafe { gsd_sizeof_type(i) };

            assert_eq!(rust_ver, c_ver);
        }
    }
}
