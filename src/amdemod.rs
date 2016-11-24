pub struct SquaringAMDemodulator<'a> {
    iterator: Box<Iterator<Item=f32> + 'a>,
}

impl<'a> SquaringAMDemodulator<'a> {
    pub fn from<I>(iterator1: I) -> SquaringAMDemodulator<'a> where I: Iterator<Item=f32> + 'a {
        SquaringAMDemodulator {
            iterator: Box::new(iterator1),
        }
    }
}

impl<'a> Iterator for SquaringAMDemodulator<'a> {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        match self.iterator.next() {
            Some(x) => Some((x * x).sqrt()),
            None => None
        }
    }
}
