const SYNC_LENGHT : usize = 40;
const SYNCA_SEQ : [bool; 40] = [false, false, false, false,
                                true, true, false, false,   // Pulse 1
                                true, true, false, false,   // Pulse 2
                                true, true, false, false,   // Pulse 3
                                true, true, false, false,   // Pulse 4
                                true, true, false, false,   // Pulse 5
                                true, true, false, false,   // Pulse 6
                                true, true, false, false,   // Pulse 7
                                false, false, false, false,
                                false, false, false, false,];

const SYNCB_SEQ : [bool; 40] = [false, false, false, false,
                                true, true, true, false, false,
                                true, true, true, false, false,
                                true, true, true, false, false,
                                true, true, true, false, false,
                                true, true, true, false, false,
                                true, true, true, false, false,
                                true, true, true, false, false,
                                false];


pub enum SyncedSample {
    Sample(f32),
    SyncA(f32),
    SyncB(f32)
}

pub struct APTSyncer<'a> {
    state: [f32; SYNC_LENGHT],
    pos : usize,
    nones_read: usize,
    max_level : f32,
    iterator: Box<Iterator<Item=f32> + 'a>
}

impl<'a> APTSyncer<'a> {
    pub fn from<I>(mut iterator: I) -> APTSyncer<'a> where I: Iterator<Item=f32> + 'a {
        let mut state = [0.0; SYNC_LENGHT];
        let mut max_level = 0.0;
        for i in 0..SYNC_LENGHT {
            match iterator.next() {
                Some(x) => {
                                state[i] = x;
                                max_level = f32::max(x, max_level);
                            },
                None => panic!("Could not retrieve enough samples to prime syncer")
            }
        }

        APTSyncer {
            state: state,
            pos: 0,
            nones_read: 0,
            max_level: max_level,
            iterator: Box::new(iterator)
        }
    }

    fn is_marker(&mut self) -> (bool, bool) {
        let mut is_a = true;
        let mut is_b = true;
        for i in 0..SYNC_LENGHT {
            let sync_pos = (self.pos + i) % SYNC_LENGHT;
            let sample = self.state[sync_pos] / self.max_level;
            is_a = is_a && ((sample > 0.5 && SYNCA_SEQ[i]) || (sample <= 0.5 && !SYNCA_SEQ[i]));
            is_b = is_b && ((sample > 0.5 && SYNCB_SEQ[i]) || (sample <= 0.5 && !SYNCB_SEQ[i]));

            if !is_a && !is_b {
                break;
            }
        }

        return (is_a, is_b);
    }
}

impl<'a> Iterator for APTSyncer<'a> {
    type Item = SyncedSample;

    fn next(&mut self) -> Option<Self::Item> {

        let (is_a, is_b) = self.is_marker();

        let sample = self.state[self.pos];
        match self.iterator.next() {
            Some(x) => {
                            self.state[self.pos] = x;
                            self.max_level = f32::max(x, self.max_level);
                        },
            None => self.nones_read += 1
        };

        if self.nones_read >= SYNC_LENGHT {
            return None;
        }

        self.pos = (self.pos + 1) % SYNC_LENGHT;

        if is_a {
            return Some(SyncedSample::SyncA(sample));
        }
        else if is_b {
            return Some(SyncedSample::SyncB(sample));
        }
        else {
            return Some(SyncedSample::Sample(sample));
        }
    }
}
