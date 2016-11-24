pub struct Mixer<'a> {
    iterator1: Box<Iterator<Item=f32> + 'a>,
    iterator2: Box<Iterator<Item=f32> + 'a>
}

impl<'a> Mixer<'a> {
    pub fn from<I,L>(iterator1: I, iterator2: L) -> Mixer<'a>
        where I: Iterator<Item=f32> + 'a, L: Iterator<Item=f32> + 'a {
        Mixer {
            iterator1: Box::new(iterator1),
            iterator2: Box::new(iterator2)
        }
    }
}

impl<'a> Iterator for Mixer<'a> {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        let val1 = match self.iterator1.next() {
            Some(x) => x,
            None => return None
        };

        let val2 = match self.iterator2.next() {
            Some(x) => x,
            None => return None
        };

        return Some(val1 * val2);
    }
}
