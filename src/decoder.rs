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

pub fn decode<T>(input_file: &str, output_file: &str, progress_update: T) -> Result<(), String>
where
    T: Fn(f32, image::GrayImage) -> bool,
{
    let mut reader = hound::WavReader::open(input_file)
        .map_err(|err| format!("Could not open inputfile: {}", err))?;

    if reader.spec().channels != 1 {
        panic!("Expected a mono file");
    }

    let sample_rate = reader.spec().sample_rate;
    println!("Samplerate: {}", sample_rate);
    if sample_rate != 48000 {
        return Err("Expected a 48kHz sample rate".to_owned());
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
            SyncedSample::SyncA(s) => {
                if !has_sync {
                    max_level = 0.0;
                    has_sync = true;
                }
                x = 0;
                s
            }
            SyncedSample::SyncB(s) => {
                if x < (PIXELS_PER_LINE / 2) {
                    let skip_distance = (PIXELS_PER_LINE / 2) - x;
                    let color = (previous_sample / max_level * 255.0) as u8;
                    for i in 0..skip_distance {
                        img.put_pixel(x + i, y, image::Luma([color]));
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
            img.put_pixel(x, y, image::Luma([color]));
        }

        x += 1;
        if x >= PIXELS_PER_LINE {
            x = 0;
            y += 1;
        }

        previous_sample = sample;

        if progress % LINES_PER_SECOND == 0 {
            if !progress_update((progress as f32) / (step * 10) as f32, img.clone()) {
                return Ok(());
            }
        }
    }
    println!("");

    progress_update(1.0, img.clone());

    img.save_with_format(&Path::new(output_file), image::ImageFormat::Png)
        .map_err(|err| format!("Could not save outputfile: {}", err))?;

    println!("Done !");

    Ok(())
}
