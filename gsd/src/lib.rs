//
// lib.rs
// Copyright (C) 2019 Malcolm Ramsay <malramsay64@gmail.com>
// Distributed under terms of the MIT license.
//

use simple_error::{bail, SimpleResult};
use std::cell::UnsafeCell;
use std::ffi::{c_void, CString};
use std::mem::MaybeUninit;

mod gsd_bindings {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    #![allow(dead_code)]
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

    pub type GSDHandle = gsd_handle;
    pub type GSDIndexEntry = gsd_index_entry;
}

use gsd_bindings::*;

pub struct GSDFrame {
    pub timestep: u64,
    pub position: Vec<[f32; 3]>,
    pub orientation: Vec<[f32; 4]>,
    pub image: Vec<[i32; 3]>,
    pub simulation_cell: [f32; 6],
}

impl GSDFrame {
    fn new(n: usize) -> GSDFrame {
        GSDFrame {
            timestep: 0,
            position: vec![[0.; 3]; n],
            orientation: vec![[0.; 4]; n],
            image: vec![[0; 3]; n],
            simulation_cell: [0.; 6],
        }
    }

    pub fn len(&self) -> usize {
        self.position.len()
    }
}

pub struct GSDTrajectory {
    curr: u64,
    file_handle: UnsafeCell<GSDHandle>,
}

impl GSDTrajectory {
    pub fn new(filename: &str) -> GSDTrajectory {
        let fname = CString::new(filename).unwrap();
        let mut handle = MaybeUninit::<GSDHandle>::uninit();
        let handle = unsafe {
            gsd_open(
                handle.as_mut_ptr(),
                fname.as_ptr(),
                gsd_open_flag_GSD_OPEN_READONLY,
            );
            handle.assume_init()
        };

        GSDTrajectory {
            curr: 0,
            file_handle: UnsafeCell::new(handle),
        }
    }

    pub fn nframes(&self) -> u64 {
        unsafe { gsd_get_nframes(self.file_handle.get()) }
    }

    fn read_chunk<T: Sized>(&self, index: u64, name: &str, chunk: &mut [T]) -> SimpleResult<()> {
        let c_name = CString::new(name).expect("CString::new failed");
        unsafe {
            let gsd_index: *const GSDIndexEntry =
                gsd_find_chunk(self.file_handle.get(), index, c_name.as_ptr());

            let expected_size = (*gsd_index).N as usize
                * (*gsd_index).M as usize
                * gsd_sizeof_type((*gsd_index).type_ as u32) as usize;
            // Check that the sizes match up
            if expected_size != chunk.len() * std::mem::size_of::<T>() {
                bail!(
                    "Incorrect size provided for '{}',
                     expected {} x {} values of {} bytes (total {} bytes), 
                     found {} elements of  {} bytes",
                    c_name.to_str().expect("String conversion failed"),
                    (*gsd_index).N,
                    (*gsd_index).M,
                    gsd_sizeof_type((*gsd_index).type_ as u32),
                    expected_size,
                    chunk.len(),
                    std::mem::size_of::<T>()
                );
            }
            gsd_read_chunk(
                self.file_handle.get(),
                chunk as *mut [T] as *mut c_void,
                gsd_index,
            );
        }
        Ok(())
    }

    pub fn get_frame(&self, index: u64) -> SimpleResult<GSDFrame> {
        let mut num_particles = [0_u32; 1];
        self.read_chunk(index, "particles/N", &mut num_particles)?;
        let mut frame = GSDFrame::new(num_particles[0] as usize);
        let mut timestep = [0_u64; 1];
        self.read_chunk(index, "configuration/step", &mut timestep)?;
        frame.timestep = timestep[0];
        self.read_chunk(index, "configuration/box", &mut frame.simulation_cell)?;
        self.read_chunk(index, "particles/orientation", &mut frame.orientation)?;
        self.read_chunk(index, "particles/position", &mut frame.position)?;
        self.read_chunk(index, "particles/image", &mut frame.image)?;
        Ok(frame)
    }
}

impl<'a> Iterator for GSDTrajectory {
    type Item = GSDFrame;

    fn next(&mut self) -> Option<Self::Item> {
        let frame = self.get_frame(self.curr).unwrap();
        self.curr += 1;
        Some(frame)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.nframes() as usize))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}
