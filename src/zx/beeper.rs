use std::collections::VecDeque;
use zx::constants::{SAMPLE_RATE, FPS};

pub type SoundSample = i16;
const SAMPLES: usize = SAMPLE_RATE / FPS;

/// transforms ear + mic to sample
fn earmic_to_sample(ear: bool, mic: bool) -> i16 {
    if ear || mic {
        i16::max_value()
    } else {
        i16::min_value()
    }
}

/// ZX Spectrum Beeper
pub struct ZXBeeper {
    ring_buffer: VecDeque<SoundSample>,
    last_pos: usize,
    ear: bool,
    mic: bool,
    last_sample: i16,
}

impl ZXBeeper {
    /// creates new ZXBeeper
    pub fn new() -> ZXBeeper {
        ZXBeeper {
            ring_buffer: VecDeque::new(),
            last_pos: 0,
            ear: false,
            mic: false,
            last_sample: 0,
        }
    }
    /// validates state of buffer.
    /// `frame_time` - value from 0 to 1, time of state change in percents
    pub fn validate(&mut self, ear: bool, mic: bool, frame_time: f64) {
        let mut next_pos = (frame_time * SAMPLES as f64) as usize;
        if next_pos > SAMPLES {
            next_pos = SAMPLES
        };
        if next_pos > self.last_pos {
            for _ in self.last_pos..next_pos {
                self.ring_buffer.push_back(self.last_sample);
            }
            self.last_pos = next_pos;
            self.last_sample = earmic_to_sample(ear, mic);
        }
    }

    /// fills all buffer
    pub fn fill_to_end(&mut self) {
        let last_pos = self.last_pos;
         self.last_pos = 0;
        if self.last_pos >= SAMPLES {
            return;
        } else {
            for _ in last_pos..SAMPLES {
                self.ring_buffer.push_back(self.last_sample);
            }
        }
    }

    /// returns last sample or `None` if queue is empty
    pub fn pop(&mut self) -> Option<SoundSample> {
        self.ring_buffer.pop_front()
    }
}
