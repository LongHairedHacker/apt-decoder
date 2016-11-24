extern crate hound;
extern crate image;

mod utils;
mod sinegen;
mod firfilter;
mod resamplers;
mod mixer;
mod amdemod;

use std::fs::File;
use std::path::Path;

use utils::float_sample_iterator;
use firfilter::FIRFilter;
use amdemod::SquaringAMDemodulator;
use resamplers::{Upsampler, Downsampler};

const LINES_PER_SECOND: u32 = 2;
const PIXELS_PER_LINE: u32 = 2080;

fn main() {
    let mut reader = match hound::WavReader::open("noaa19_20160814_mono.wav") {
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

    println!("{} Samples", reader.len());

    let seconds = (reader.len() as f32) / (sample_rate as f32);
    let lines = (seconds.ceil() as u32) * LINES_PER_SECOND;
    println!("{} or {} lines", seconds, lines);

    let mut img = image::ImageBuffer::new(PIXELS_PER_LINE, lines);



    let samples = float_sample_iterator(&mut reader);

    let coeffs = vec![ -7.383784e-03,
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
        -7.383784e-03];


    let demod = SquaringAMDemodulator::from(samples);
    let filter = FIRFilter::from(demod, coeffs);
    let upsampler = Upsampler::from(filter, 13);
    let downsampler = Downsampler::from(upsampler, 150);

    let mut x = 0;
    let mut y = 0;
    let mut max_level = 0.0;
    for sample in downsampler {
        max_level = f32::max(sample, max_level);
        let color = (sample / max_level * 255.0) as u8;

        println!("{}", color);

        img.put_pixel(x,y,image::Luma([color]));

        x += 1;
        if x > PIXELS_PER_LINE {
            x = 0;
            y += 1;
        }
    }

    let ref mut fout = File::create(&Path::new("decoded.png")).unwrap();
    image::ImageLuma8(img).save(fout, image::PNG).unwrap();
}
