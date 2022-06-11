//
// build.rs
// Copyright (C) 2019 Malcolm Ramsay <malramsay64@gmail.com>
// Distributed under terms of the MIT license.
//

use std::env;
use std::path::PathBuf;

fn main() {
    cc::Build::new()
        .file("gsd_c/gsd/gsd.c")
        .extra_warnings(false)
        .compile("gsd");

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("gsd_c/gsd/gsd.h")
        // Whitelist the functions and types which are required
        .allowlist_type("gsd_index_entry")
        .allowlist_type("gsd_handle")
        .allowlist_function("gsd_open")
        .allowlist_function("gsd_close")
        .allowlist_function("gsd_get_nframes")
        .allowlist_function("gsd_read_chunk")
        .allowlist_function("gsd_find_chunk")
        .allowlist_function("gsd_sizeof_type")
        .derive_debug(true)
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
