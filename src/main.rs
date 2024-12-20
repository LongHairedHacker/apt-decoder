#![windows_subsystem = "windows"]

extern crate clap;
extern crate hound;
extern crate image;
extern crate indicatif;
extern crate rfd;
extern crate thiserror;

#[cfg(feature = "ui")]
extern crate eframe;

mod amdemod;
mod aptsyncer;
mod cli;
mod decoder;
mod errors;
mod firfilter;
mod resamplers;
mod utils;

use clap::{arg, command};

#[cfg(not(feature = "ui"))]
fn main() {
    let matches = command!()
        .arg(arg!([wavfile] "Input wav file with 48kHz samplingrate").required(true))
        .arg(arg!([pngfile] "Output png file").default_value("output.png"))
        .get_matches();

    let input_file = matches
        .get_one::<String>("wavfile")
        .expect("No input file given");

    let output_file = matches
        .get_one::<String>("pngfile")
        .expect("No output file given");

    cli::decode(&input_file, &output_file);
}

#[cfg(feature = "ui")]
mod ui;

#[cfg(feature = "ui")]
use ui::DecoderApp;

#[cfg(feature = "ui")]
fn main() {
    let matches = command!()
        .arg(arg!([wavfile] "Input wav file with 48kHz samplingrate").default_value("input.wav"))
        .arg(arg!([pngfile] "Output png file").default_value("output.png"))
        .arg(arg!(-n --nogui "Disable gui and run in command line mode"))
        .get_matches();

    let input_file = matches
        .get_one::<String>("wavfile")
        .expect("No input file given")
        .to_string();
    let output_file = matches
        .get_one::<String>("pngfile")
        .expect("No output file given")
        .to_string();

    if matches.get_flag("nogui") {
        cli::decode(&input_file, &output_file);
    } else {
        let native_options = eframe::NativeOptions::default();

        eframe::run_native(
            "APT-Decoder",
            native_options,
            Box::new(move |_cc| Ok(Box::new(DecoderApp::new(&input_file, &output_file)))),
        )
        .unwrap();
    }
}
