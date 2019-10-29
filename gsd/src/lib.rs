//
// lib.rs
// Copyright (C) 2019 Malcolm Ramsay <malramsay64@gmail.com>
// Distributed under terms of the MIT license.
//

use anyhow::{anyhow, bail, Error};
use std::cell::UnsafeCell;
use std::ffi::{c_void, CString};
use std::mem::MaybeUninit;
use std::path::Path;

mod gsd_bindings;

use gsd_bindings::*;

#[derive(Clone, Debug)]
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

    pub fn is_empty(&self) -> bool {
        self.position.is_empty()
    }
}

/// A handle to a GSD Trajectory allowing interaction
///
/// This provides a handle to interact with a GSD file, providing utilties to read individual
/// frames in addition to being able to iterate over the entire trajectory. This provides a safe
/// wrapper to the `gsd_open` funnction.
pub struct GSDTrajectory {
    curr: u64,
    // The handle reuqires many mutable references, so the UnsafeCell construct is the most
    // sensible for this use case. Additionally it doesn't support Sync so handling a trajecotry
    // in multiple threads is currently unsupported.
    file_handle: UnsafeCell<GSDHandle>,
}

impl GSDTrajectory {
    pub fn new<P: AsRef<Path>>(filename: P) -> Result<GSDTrajectory, Error> {
        let fname = CString::new(
            filename
                .as_ref()
                .to_str()
                .ok_or_else(|| anyhow!("Unable to convert filename to str"))?,
        )?;
        let mut handle = MaybeUninit::<GSDHandle>::uninit();
        let retvalue = unsafe {
            gsd_open(
                handle.as_mut_ptr(),
                fname.as_ptr(),
                gsd_open_flag_GSD_OPEN_READONLY,
            )
        };
        // Check return value
        let handle = match retvalue {
            // Opening file succeeded, assume handle is initialised
            // Sucess
            0 => unsafe { handle.assume_init() },
            -1 => bail!("IO Error"),
            -2 => bail!("Not a GSD File"),
            -3 => bail!("Invalid GSD version"),
            -4 => bail!("File has been corrupted"),
            -5 => bail!("Internal error, unable to allocate memory."),
            _ => bail!("Unknown error opening file."),
        };

        Ok(GSDTrajectory {
            curr: 0,
            file_handle: UnsafeCell::new(handle),
        })
    }

    pub fn nframes(&self) -> u64 {
        unsafe { gsd_get_nframes(self.file_handle.get()) }
    }

    fn _safe_gsd_find_chunk(&self, frame: u64, name: &str) -> Result<GSDIndexEntry, Error> {
        let c_name = CString::new(name)?;
        unsafe { gsd_find_chunk(self.file_handle.get(), frame, c_name.as_ptr()).as_ref() }
            .cloned()
            .ok_or_else(|| anyhow!("Chunk '{}' was not found", name))
    }

    fn read_chunk<T: Sized>(&self, index: u64, name: &str, chunk: &mut [T]) -> Result<(), Error> {
        let gsd_index = match self._safe_gsd_find_chunk(index, name) {
            Ok(g) => g,
            Err(_) => return Ok(()),
        };

        // This checks that we are going to read the input correctly and produces a useful error
        // message should there be a mismatch of sizes.
        if gsd_index.expected_size()? != chunk.len() * std::mem::size_of::<T>() {
            bail!(
                "Incorrect size provided for '{}',
                 expected {} x {} values of {} bytes (total {} bytes), 
                 found {} elements of  {} bytes",
                name,
                gsd_index.N,
                gsd_index.M,
                gsd_index.type_size()?,
                gsd_index.expected_size()?,
                chunk.len(),
                std::mem::size_of::<T>()
            );
        }

        let returnval = unsafe {
            gsd_read_chunk(
                self.file_handle.get(),
                chunk as *mut [T] as *mut c_void,
                &gsd_index as *const GSDIndexEntry,
            )
        };

        match returnval {
            0 => Ok(()),
            -2 => Err(anyhow!("Invalid Input")),
            -1 => Err(anyhow!("IO Failure")),
            _ => Err(anyhow!("Unknown Error")),
        }
    }

    pub fn get_frame(&self, index: u64) -> Result<GSDFrame, Error> {
        let mut num_particles = [0_u32; 1];
        self.read_chunk(index, "particles/N", &mut num_particles)?;
        let mut frame = GSDFrame::new(num_particles[0] as usize);
        let mut timestep = [0_u64; 1];
        self.read_chunk(index, "configuration/step", &mut timestep)?;
        frame.timestep = timestep[0];
        // These are required components
        self.read_chunk(index, "configuration/box", &mut frame.simulation_cell)?;
        self.read_chunk(index, "particles/orientation", &mut frame.orientation)?;
        self.read_chunk(index, "particles/position", &mut frame.position)?;

        // These are optional components
        self.read_chunk(index, "particles/image", &mut frame.image)
            .unwrap_or(());

        Ok(frame)
    }
}

impl Drop for GSDTrajectory {
    fn drop(&mut self) {
        unsafe { gsd_close(self.file_handle.get()) };
    }
}

impl<'a> Iterator for GSDTrajectory {
    type Item = GSDFrame;

    fn next(&mut self) -> Option<Self::Item> {
        self.curr += 1;
        match self.get_frame(self.curr - 1) {
            Ok(frame) => Some(frame),
            Err(_) if self.curr >= self.nframes() => None,
            Err(e) => {
                println!("{}", e);
                None
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.nframes() as usize))
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.curr += 1 + n as u64;
        match self.get_frame(self.curr - 1) {
            Ok(frame) => Some(frame),
            Err(_) if self.curr >= self.nframes() => None,
            Err(e) => {
                println!("{}", e);
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn file_read() {
        let mut filename = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        filename.push("tests");
        filename.push("trajectory.gsd");
        println!("Filename: {:?}", &filename);
        GSDTrajectory::new(filename).unwrap();
    }
}
