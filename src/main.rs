extern crate eframe;
extern crate hound;
extern crate image;
extern crate rfd;

mod amdemod;
mod aptsyncer;
mod firfilter;
mod resamplers;
mod utils;

use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time;

use amdemod::SquaringAMDemodulator;
use aptsyncer::{APTSyncer, SyncedSample};
use firfilter::FIRFilter;
use resamplers::{Downsampler, Upsampler};
use utils::float_sample_iterator;

const LINES_PER_SECOND: u32 = 2;
const PIXELS_PER_LINE: u32 = 2080;

const LOWPASS_COEFFS: [f32; 63] = [
    -7.383784e-03,
    -3.183046e-03,
    2.255039e-03,
    7.461166e-03,
    1.091908e-02,
    1.149109e-02,
    8.769802e-03,
    3.252932e-03,
    -3.720606e-03,
    -1.027446e-02,
    -1.447403e-02,
    -1.486427e-02,
    -1.092423e-02,
    -3.307958e-03,
    6.212477e-03,
    1.511364e-02,
    2.072873e-02,
    2.096037e-02,
    1.492345e-02,
    3.347624e-03,
    -1.138407e-02,
    -2.560252e-02,
    -3.507114e-02,
    -3.591225e-02,
    -2.553830e-02,
    -3.371569e-03,
    2.882645e-02,
    6.711368e-02,
    1.060042e-01,
    1.394643e-01,
    1.620650e-01,
    1.700462e-01,
    1.620650e-01,
    1.394643e-01,
    1.060042e-01,
    6.711368e-02,
    2.882645e-02,
    -3.371569e-03,
    -2.553830e-02,
    -3.591225e-02,
    -3.507114e-02,
    -2.560252e-02,
    -1.138407e-02,
    3.347624e-03,
    1.492345e-02,
    2.096037e-02,
    2.072873e-02,
    1.511364e-02,
    6.212477e-03,
    -3.307958e-03,
    -1.092423e-02,
    -1.486427e-02,
    -1.447403e-02,
    -1.027446e-02,
    -3.720606e-03,
    3.252932e-03,
    8.769802e-03,
    1.149109e-02,
    1.091908e-02,
    7.461166e-03,
    2.255039e-03,
    -3.183046e-03,
    -7.383784e-03,
];

use eframe::egui::text_edit::TextEdit;
use eframe::egui::widgets::{Button, ProgressBar};
use eframe::egui::{Color32, RichText};
use eframe::{egui, epi};

#[derive(PartialEq)]
enum DecoderRunState {
    RUNNING,
    CANCELED,
    DONE,
}

struct DecoderJobState {
    progress: f32,
    run_state: DecoderRunState,
}

impl DecoderJobState {
    fn is_running(&self) -> bool {
        self.run_state == DecoderRunState::RUNNING
    }
}

impl Default for DecoderJobState {
    fn default() -> Self {
        Self {
            progress: 0.0,
            run_state: DecoderRunState::DONE,
        }
    }
}

pub struct DecoderApp {
    input_path: String,
    output_path: String,
    decoding_state: Arc<Mutex<DecoderJobState>>,
}

impl Default for DecoderApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            input_path: "input.wav".to_owned(),
            output_path: "output.png".to_owned(),
            decoding_state: Arc::new(Mutex::new(DecoderJobState::default())),
        }
    }
}

impl epi::App for DecoderApp {
    fn name(&self) -> &str {
        "eframe template"
    }

    /// Called once before the first frame.
    fn setup(
        &mut self,
        _ctx: &egui::CtxRef,
        _frame: &epi::Frame,
        _storage: Option<&dyn epi::Storage>,
    ) {
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::CtxRef, frame: &epi::Frame) {
        let Self {
            input_path,
            output_path,
            decoding_state,
        } = self;

        {
            let mut state = decoding_state.lock().unwrap();

            if !ctx.input().raw.dropped_files.is_empty() && !state.is_running() {
                if let Some(path) = ctx.input().raw.dropped_files[0].clone().path {
                    *input_path = path.display().to_string();
                }
            }

            egui::CentralPanel::default().show(ctx, |ui| {
                ui.heading("APT-Decoder");

                egui::Grid::new("form_grid").num_columns(3).show(ui, |ui| {
                    ui.label("Input Wav File:");
                    ui.add_sized([300.0, 20.0], TextEdit::singleline(input_path));

                    if ui
                        .add_enabled(!state.is_running(), Button::new("Open"))
                        .clicked()
                    {
                        if let Some(path) = rfd::FileDialog::new().pick_file() {
                            *input_path = path.display().to_string();
                        }
                    };
                    ui.end_row();

                    ui.label("Output PNG File:");
                    ui.add_sized([300.0, 20.0], TextEdit::singleline(output_path));
                    if ui
                        .add_enabled(!state.is_running(), Button::new("Save"))
                        .clicked()
                    {
                        if let Some(path) = rfd::FileDialog::new().save_file() {
                            *output_path = path.display().to_string();
                        }
                    };
                    ui.end_row();
                });

                ui.horizontal(|ui| {
                    if ui
                        .add_enabled(!state.is_running(), Button::new("Decode"))
                        .clicked()
                    {
                        let frame = frame.clone();
                        let decoding_state = decoding_state.clone();
                        state.run_state = DecoderRunState::RUNNING;
                        let thread = std::thread::spawn(move || {
                            for i in 0..101 {
                                {
                                    let mut state = decoding_state.lock().unwrap();
                                    if state.run_state == DecoderRunState::CANCELED {
                                        frame.request_repaint();
                                        return;
                                    }
                                    state.progress = (i as f32) / 100.0;
                                }
                                frame.request_repaint();

                                std::thread::sleep(time::Duration::from_millis(200));
                            }

                            {
                                let mut state = decoding_state.lock().unwrap();
                                state.progress = 100.0;
                                state.run_state = DecoderRunState::DONE;
                            }
                        });
                    }
                    if ui
                        .add_enabled(
                            state.is_running(),
                            Button::new(RichText::new("Cancel").color(Color32::RED)),
                        )
                        .clicked()
                    {
                        state.run_state = DecoderRunState::CANCELED;
                    }
                });

                let progressbar = ProgressBar::new(state.progress).show_percentage();
                ui.add(progressbar);

                ui.separator();
            });
        }
    }
}

fn main() {
    let app = DecoderApp::default();
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(app), native_options);

    /*
        let args : Vec<String> = std::env::args().collect();

        if args.len() != 3 {
            println!("Usage: {} <wav file> <output file>", args[0]);
            return;
        }

        let mut reader = match hound::WavReader::open(&args[1]) {
            Err(e) => panic!("Could not open inputfile: {}", e),
            Ok(r) => r
        };

        if reader.spec().channels != 1 {
            panic!("Expected a mono file");
        }

        let sample_rate = reader.spec().sample_rate;
        println!("Samplerate: {}", sample_rate);
        if sample_rate != 48000 {
            panic!("Expected a 48kHz sample rate");
        }

        let sample_count = reader.len();
        let seconds = (sample_count as f32) / (sample_rate as f32);
        let lines = (seconds.ceil() as u32) * LINES_PER_SECOND;
        println!("File contains {} seconds or {} lines", seconds, lines);

        let mut img = image::ImageBuffer::new(PIXELS_PER_LINE, lines);

        let coeffs = &LOWPASS_COEFFS;

        let samples = float_sample_iterator(&mut reader);

        let demod = SquaringAMDemodulator::from(samples);
        let filter = FIRFilter::from(demod, coeffs);
        let upsampler = Upsampler::from(filter, 13);
        let downsampler = Downsampler::from(upsampler, 150);
        let syncer = APTSyncer::from(downsampler);

        let mut x = 0;
        let mut y = 0;
        let mut max_level = 0.0;
        let mut has_sync = false;

        let mut progress = 0;
        let step = sample_count * 13 / 150 / 10;

        let mut previous_sample = 0.0;

        print!("0%");
        std::io::stdout().flush().unwrap();
        for synced_sample in syncer {
            progress += 1;

            if progress % step == 0 {
                print!("...{}%", progress / step * 10);
                std::io::stdout().flush().unwrap();
            }

            let sample = match synced_sample {
                SyncedSample::Sample(s) => s,
                SyncedSample::SyncA(s) =>{
                    if !has_sync {
                        max_level = 0.0;
                        has_sync = true;
                    }
                    x = 0;
                    s
                }
                SyncedSample::SyncB(s) =>{
                    if x < (PIXELS_PER_LINE / 2) {
                        let skip_distance = (PIXELS_PER_LINE / 2) - x;
                        let color = (previous_sample / max_level * 255.0) as u8;
                        for i in 0..skip_distance {
                            img.put_pixel(x + i,y,image::Luma([color]));
                        }
                    }
                    if !has_sync {
                        max_level = 0.0;
                        has_sync = true;
                    }
                    x = PIXELS_PER_LINE / 2;
                    s
                }
            };

            max_level = f32::max(sample, max_level);
            let color = (sample / max_level * 255.0) as u8;

            if y < lines {
                img.put_pixel(x,y,image::Luma([color]));
            }

            x += 1;
            if x >= PIXELS_PER_LINE {
                x = 0;
                y += 1;
            }

            previous_sample = sample;
        }
        println!("");

        let ref mut fout = match File::create(&Path::new(&args[2])) {
            Err(e) => panic!("Could not open outputfile: {}", e),
            Ok(f) => f
        };

        image::ImageLuma8(img).save(fout, image::PNG).unwrap();
    */
    println!("Done !");
}
