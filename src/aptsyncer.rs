const SYNC_LENGHT: usize = 40;
const SYNCA_SEQ: [bool; 40] = [
    false, false, false, false, true, true, false, false, // Pulse 1
    true, true, false, false, // Pulse 2
    true, true, false, false, // Pulse 3
    true, true, false, false, // Pulse 4
    true, true, false, false, // Pulse 5
    true, true, false, false, // Pulse 6
    true, true, false, false, // Pulse 7
    false, false, false, false, false, false, false, false,
];

const SYNCB_SEQ: [bool; 40] = [
    false, false, false, false, true, true, true, false, false, true, true, true, false, false,
    true, true, true, false, false, true, true, true, false, false, true, true, true, false, false,
    true, true, true, false, false, true, true, true, false, false, false,
];

pub enum SyncedSample {
    Sample(f32),
    SyncA(f32),
    SyncB(f32),
}

pub struct APTSyncer<'a> {
    state: [f32; SYNC_LENGHT],
    pos: usize,
    nones_read: usize,
    avg_level: f32,
    iterator: Box<dyn Iterator<Item = f32> + 'a>,
}

impl<'a> APTSyncer<'a> {
    pub fn from<I>(mut iterator: I) -> APTSyncer<'a>
    where
        I: Iterator<Item = f32> + 'a,
    {
        let mut state = [0.0; SYNC_LENGHT];
        let mut avg_level = 0.5;
        for i in 0..SYNC_LENGHT {
            match iterator.next() {
                Some(x) => {
                    state[i] = x;
                    avg_level = 0.25 * x + avg_level * 0.75;
                }
                None => panic!("Could not retrieve enough samples to prime syncer"),
            }
        }

        APTSyncer {
            state: state,
            pos: 0,
            nones_read: 0,
            avg_level: avg_level,
            iterator: Box::new(iterator),
        }
    }

    fn is_marker(&mut self) -> (bool, bool) {
        let mut count_a = 0;
        let mut count_b = 0;

        for i in 0..SYNC_LENGHT {
            let sync_pos = (self.pos + i) % SYNC_LENGHT;
            let sample = self.state[sync_pos] / (self.avg_level * 2.0);
            if (sample > 0.5 && SYNCA_SEQ[i]) || (sample <= 0.5 && !SYNCA_SEQ[i]) {
                count_a += 1;
            }
            if (sample > 0.5 && SYNCB_SEQ[i]) || (sample <= 0.5 && !SYNCB_SEQ[i]) {
                count_b += 1;
            }

            /*
            if !count_a && !count_b {
                break;
            }
            */
        }

        return (count_a > 35, count_b > 35);
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
                self.avg_level = 0.25 * x + self.avg_level * 0.75;
            }
            None => self.nones_read += 1,
        };

        if self.nones_read >= SYNC_LENGHT {
            return None;
        }

        self.pos = (self.pos + 1) % SYNC_LENGHT;

        if is_a {
            return Some(SyncedSample::SyncA(sample));
        } else if is_b {
            return Some(SyncedSample::SyncB(sample));
        } else {
            return Some(SyncedSample::Sample(sample));
        }
    }
}
