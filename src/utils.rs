use std;

extern crate hound;

type FileReader = std::io::BufReader<std::fs::File>;

pub fn float_sample_iterator<'a>(
    reader: &'a mut hound::WavReader<FileReader>,
) -> Box<dyn Iterator<Item = f32> + 'a> {
    match reader.spec().sample_format {
        hound::SampleFormat::Float => Box::new(reader.samples::<f32>().map(|x| x.unwrap())),
        hound::SampleFormat::Int => match reader.spec().bits_per_sample {
            8 => Box::new(
                reader
                    .samples::<i8>()
                    .map(|x| (x.unwrap() as f32) / (i16::max_value() as f32)),
            ),
            16 => Box::new(
                reader
                    .samples::<i16>()
                    .map(|x| (x.unwrap() as f32) / (i16::max_value() as f32)),
            ),
            32 => Box::new(
                reader
                    .samples::<i32>()
                    .map(|x| (x.unwrap() as f32) / (i32::max_value() as f32)),
            ),
            _ => panic!("Unsupported sample format"),
        },
    }
}
