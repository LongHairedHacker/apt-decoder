use std;

pub struct SineGenerator {
    freq: f32,
    amplitude: f32,
    sample_freq: f32,
    phase: f32
}

impl SineGenerator {
    pub fn new(freq: f32, amplitude: f32, sample_freq: f32) -> SineGenerator {
        SineGenerator {
            freq: freq,
            amplitude: amplitude,
            sample_freq: sample_freq,
            phase: 0.0
        }
    }
}

impl Iterator for SineGenerator {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        let result = self.amplitude * self.phase.sin();
        self.phase += 2.0 *  std::f32::consts::PI * self.freq / self.sample_freq;

        Some(result)
    }
}
