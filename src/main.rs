extern crate hound;

fn float_sample_iterator<'a>(reader: &'a mut hound::WavReader<std::io::BufReader<std::fs::File>>)
    -> Box<Iterator<Item=f32> + 'a> {
    match reader.spec().sample_format {
        hound::SampleFormat::Float => Box::new(reader.samples::<f32>().map(|x| x.unwrap())),
        hound::SampleFormat::Int => match reader.spec().bits_per_sample {
            8 => Box::new(reader.samples::<i8>().map(|x| (x.unwrap() as f32) / (i16::max_value() as f32))),
            16 => Box::new(reader.samples::<i16>().map(|x| (x.unwrap() as f32) / (i16::max_value() as f32))),
            32 => Box::new(reader.samples::<i32>().map(|x| (x.unwrap() as f32) / (i32::max_value() as f32))),
            _ => panic!("Unsupported sample rate")
        }
    }
}


fn main() {
    let carrier_freq = 2400;

    let mut reader = match hound::WavReader::open("noaa19_short.wav") {
        Err(e) => panic!("Could not open inputfile: {}", e),
        Ok(r) => r
    };

    if reader.spec().channels != 1 {
        panic!("Expected a mono file");
    }

    let sample_rate = reader.spec().sample_rate;
    println!("Samplerate: {}", sample_rate);

    let mut samples = float_sample_iterator(&mut reader);

    for sample in samples {
        println!("Sample: {}", sample )
    }
}
