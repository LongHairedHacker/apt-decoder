pub struct FIRFilter<'a> {
    coeffs: &'a [f32],
    state: Vec<f32>,
    pos: usize,
    iterator: Box<dyn Iterator<Item = f32> + 'a>,
}

impl<'a> FIRFilter<'a> {
    pub fn from<I>(iterator: I, coeffs: &'a [f32]) -> FIRFilter<'a>
    where
        I: Iterator<Item = f32> + 'a,
    {
        let mut state = Vec::new();
        for _ in 0..coeffs.len() {
            state.push(0.0);
        }

        FIRFilter {
            coeffs: coeffs,
            state: state,
            pos: 0,
            iterator: Box::new(iterator),
        }
    }
}

impl<'a> Iterator for FIRFilter<'a> {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        let cur = match self.iterator.next() {
            Some(x) => x,
            None => return None,
        };

        self.pos = (self.pos + 1) % self.coeffs.len();
        self.state[self.pos] = cur;

        let mut result = 0.0;
        for i in 0..self.coeffs.len() {
            let pos = (self.pos + self.coeffs.len() - i) % self.coeffs.len();
            result += self.state[pos] * self.coeffs[i];
        }

        Some(result)
    }
}
