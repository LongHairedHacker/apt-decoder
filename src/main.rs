extern crate hound;

type FileReader = std::io::BufReader<std::fs::File>;

fn float_sample_iterator<'a>(reader: &'a mut hound::WavReader<FileReader>)
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

struct Upsampler<'a> {
    factor: u16,
    state: u16,
    iterator: Box<Iterator<Item=f32> + 'a>
}

impl<'a> Upsampler<'a> {
    fn from<I>(iterator: I, factor: u16) -> Upsampler<'a> where I: Iterator<Item=f32> + 'a {
        Upsampler {
            factor: factor,
            state: 0,
            iterator: Box::new(iterator)
        }
    }
}

impl<'a> Iterator for Upsampler<'a> {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        let result = if self.state == 0 {
            self.iterator.next()
        }
        else {
            Some(0.0)
        };
        self.state = (self.state + 1) % self.factor;

        return result;
    }
}



struct Downsampler<'a> {
        factor: u16,
        iterator: Box<Iterator<Item=f32> + 'a>
}

impl<'a> Downsampler<'a> {
    fn from<I>(iterator: I, factor: u16) -> Downsampler<'a> where I: Iterator<Item=f32> + 'a {
        Downsampler {
            factor: factor,
            iterator: Box::new(iterator)
        }
    }
}

impl<'a> Iterator for Downsampler<'a> {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        let mut result = 0.0;
        for _ in 0..self.factor {
            match self.iterator.next() {
                Some(x) => result += x,
                None => return None
            }
        }
        result /= self.factor as f32;

        return Some(result);
    }
}



struct FIRFilter<'a> {
    coeffs: Vec<f32>,
    state: Vec<f32>,
    pos: usize,
    iterator: Box<Iterator<Item=f32> + 'a>
}

impl<'a> FIRFilter<'a> {
    fn from<I>(iterator: I, coeffs: Vec<f32>) -> FIRFilter<'a> where I: Iterator<Item=f32> + 'a {
        let mut state = Vec::new();
        for _ in 0..coeffs.len() {
            state.push(0.0);
        }

        FIRFilter {
            coeffs: coeffs,
            state: state,
            pos: 0,
            iterator: Box::new(iterator)
        }
    }
}

impl<'a> Iterator for FIRFilter<'a> {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        let cur = match self.iterator.next() {
            Some(x) => x,
            None => return None
        };

        self.pos = (self.pos + 1) % self.coeffs.len();
        self.state[self.pos] = cur;

        let mut result = 0.0;
        for i in 0..self.coeffs.len() {
            let pos = (self.pos + self.coeffs.len() - i) % self.coeffs.len();
            result += self.state[pos] * self.coeffs[i];
        };

        Some(result)
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

    let samples = float_sample_iterator(&mut reader);

    let coeffs = vec![1.73203081e-03,   3.68489420e-03,  -1.61573864e-03,  -4.83850760e-03,
       1.26938317e-03,   6.13073242e-03,  -6.37488600e-04,  -7.54064630e-03,
      -3.41166003e-04,   9.04137653e-03,   1.73642240e-03,  -1.06008349e-02,
      -3.63238422e-03,   1.21827130e-02,   6.13805128e-03,  -1.37477000e-02,
      -9.40748113e-03,   1.52548738e-02,   1.36795184e-02,  -1.66632054e-02,
      -1.93616657e-02,   1.79331113e-02,   2.72262912e-02,  -1.90279842e-02,
      -3.89431985e-02,   1.99156328e-02,   5.88894574e-02,  -2.05695633e-02,
      -1.03195587e-01,   2.09700453e-02,   3.17333203e-01,   4.78895090e-01,
       3.17333203e-01,   2.09700453e-02,  -1.03195587e-01,  -2.05695633e-02,
       5.88894574e-02,   1.99156328e-02,  -3.89431985e-02,  -1.90279842e-02,
       2.72262912e-02,   1.79331113e-02,  -1.93616657e-02,  -1.66632054e-02,
       1.36795184e-02,   1.52548738e-02,  -9.40748113e-03,  -1.37477000e-02,
       6.13805128e-03,   1.21827130e-02,  -3.63238422e-03,  -1.06008349e-02,
       1.73642240e-03,   9.04137653e-03,  -3.41166003e-04,  -7.54064630e-03,
      -6.37488600e-04,   6.13073242e-03,   1.26938317e-03,  -4.83850760e-03,
      -1.61573864e-03,   3.68489420e-03,   1.73203081e-03];



    let filter = FIRFilter::from(samples, coeffs);

    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 48000,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create("lowpass.wav", spec).unwrap();


    for sample in filter {
        println!("{}", sample);

        let amplitude = i32::max_value() as f32;
        writer.write_sample((sample * amplitude) as i32).unwrap();
    }
}
