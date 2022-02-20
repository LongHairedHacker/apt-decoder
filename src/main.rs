#![windows_subsystem = "windows"]

extern crate eframe;
extern crate hound;
extern crate image;
extern crate rfd;
extern crate thiserror;

mod amdemod;
mod aptsyncer;
mod decoder;
mod errors;
mod firfilter;
mod resamplers;
mod ui;
mod utils;

use ui::DecoderApp;

fn main() {
    let app = DecoderApp::default();
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(app), native_options);

    //let args: Vec<String> = std::env::args().collect();

    /*
    if args.len() != 3 {
        println!("Usage: {} <wav file> <output file>", args[0]);
        return;
    }
    */
}
