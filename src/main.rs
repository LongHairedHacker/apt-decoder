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
        .arg(
            arg!([wavfile] "Input wav file with 48kHz samplingrate")
                .required(true)
                .allow_invalid_utf8(true),
        )
        .arg(
            arg!([pngfile] "Output png file")
                .default_value("output.png")
                .allow_invalid_utf8(true),
        )
        .get_matches();

    let input_file = matches
        .value_of_os("wavfile")
        .expect("No input file given")
        .to_str()
        .unwrap();
    let output_file = matches
        .value_of_os("pngfile")
        .expect("No output file given")
        .to_str()
        .unwrap();

    cli::decode(input_file, output_file);
}

#[cfg(feature = "ui")]
mod ui;

#[cfg(feature = "ui")]
use ui::DecoderApp;

#[cfg(feature = "ui")]
fn main() {
    let matches = command!()
        .arg(
            arg!([wavfile] "Input wav file with 48kHz samplingrate")
                .default_value("input.wav")
                .allow_invalid_utf8(true),
        )
        .arg(
            arg!([pngfile] "Output png file")
                .default_value("output.png")
                .allow_invalid_utf8(true),
        )
        .arg(arg!(-n --nogui ... "Disable gui and run in command line mode"))
        .get_matches();

    let input_file = matches
        .value_of_os("wavfile")
        .expect("No input file given")
        .to_str()
        .unwrap();
    let output_file = matches
        .value_of_os("pngfile")
        .expect("No output file given")
        .to_str()
        .unwrap();

    if matches.is_present("nogui") {
        cli::decode(input_file, output_file);
    } else {
        let app = DecoderApp::new(input_file, output_file);
        let native_options = eframe::NativeOptions::default();
        eframe::run_native(Box::new(app), native_options);
    }
}
