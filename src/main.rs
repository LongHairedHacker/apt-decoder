extern crate hound;

mod utils;
mod sinegen;
mod firfilter;
mod resamplers;
mod mixer;

use utils::float_sample_iterator;
use sinegen::SineGenerator;
use firfilter::FIRFilter;
use mixer::Mixer;


fn main() {
    let carrier_freq = 2400.0;
    let sample_freq = 48000.0;

    let mut reader = match hound::WavReader::open("noaa19_short.wav") {
        Err(e) => panic!("Could not open inputfile: {}", e),
        Ok(r) => r
    };

    if reader.spec().channels != 1 {
        panic!("Expected a mono file");
    }

    let sample_rate = reader.spec().sample_rate;
    println!("Samplerate: {}", sample_rate);

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



    let sine_gen = SineGenerator::new(carrier_freq, 1.0, sample_freq);
    let mixer = Mixer::from(sine_gen, samples);
    let filter = FIRFilter::from(mixer, coeffs);

    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 48000,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create("demod.wav", spec).unwrap();


    for sample in filter {
        //println!("{}", sample);

        let amplitude = (i32::max_value() as f32) * 0.8; //About 1dB headroom
        writer.write_sample((sample * amplitude) as i32).unwrap();
    }
}
