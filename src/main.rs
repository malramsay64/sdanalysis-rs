use clap::{App, Arg};

use gsd::GSDTrajectory;

fn main() {
    let matches = App::new("GSD Parser")
        .version("0.1")
        .author("Malcolm Ramsay")
        .arg(
            Arg::with_name("filename")
                .index(1)
                .required(true)
                .help("File to process"),
        )
        .get_matches();

    let filename = matches.value_of("filename").unwrap();

    for frame in GSDTrajectory::new(filename).take(4) {
        println!("{}", frame.timestep);
    }

    println!("filename: {}", filename)
}
