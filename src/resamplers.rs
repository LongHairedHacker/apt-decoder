pub struct Upsampler<'a> {
    factor: u16,
    state: u16,
    iterator: Box<Iterator<Item=f32> + 'a>
}

impl<'a> Upsampler<'a> {
    pub fn from<I>(iterator: I, factor: u16) -> Upsampler<'a> where I: Iterator<Item=f32> + 'a {
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



pub struct Downsampler<'a> {
        factor: u16,
        iterator: Box<Iterator<Item=f32> + 'a>
}

impl<'a> Downsampler<'a> {
    pub fn from<I>(iterator: I, factor: u16) -> Downsampler<'a> where I: Iterator<Item=f32> + 'a {
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
