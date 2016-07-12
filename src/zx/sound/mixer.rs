//! Module implemets zx spectrum audio devices mixer
use std::i16;
use std::collections::VecDeque;
use zx::sound::{SAMPLES, SoundSample, samples_from_time, SampleGenerator};
use zx::sound::beeper::*;
use zx::sound::ay::*;

/// Main sound mixer.
/// TODO: parametrize
pub struct ZXMixer {
    /// direct access to beeper device
    pub beeper: ZXBeeper,
    /// direct access to AY device
    pub ay: ZXAyChip,
    ring_buffer: VecDeque<SoundSample<i16>>,
    last_pos: usize,
    last_sample: SoundSample<i16>,
    master_volume: f64,
    beeper_volume: f64,
    ay_volume: f64,
}

impl ZXMixer {
    /// Constructs new Mixer structure
    /// # Arguments
    /// - `use_beeper` - process beeper or not
    /// - `use_ay` - process ay chip or not
    pub fn new(_use_beeper: bool, _use_ay: bool ) -> ZXMixer {
        ZXMixer {
            beeper: ZXBeeper::new(),
            ay: ZXAyChip::new(),
            ring_buffer: VecDeque::with_capacity(SAMPLES),
            last_pos: 0,
            last_sample: 0,
            master_volume: 1.0,
            beeper_volume: 1.0,
            ay_volume: 1.0,
        }
    }

    fn gen_sample(&mut self) -> SoundSample<i16> {
        // TODO: find out how to mix channels equally
        let sample_float = (/*self.beeper.gen_float_sample() + */self.ay.gen_float_sample()) / 2f64
            * self.master_volume;
        let sample = (sample_float * i16::max_value() as f64) as i16;
        self.last_sample = sample;
        sample
    }
    /// Updates internal buffer of mixer and fills it with new samples
    pub fn process(&mut self, current_time: f64) {
        // buffer overflow
        if self.ring_buffer.len() >= SAMPLES {
            return;
        }
        // so at this moment we need to get new samples from devices
        let curr_pos = samples_from_time(current_time);
        // if we on same pos or frame passed then no new samples
        if curr_pos <= self.last_pos {
            return;
        }
        let sample_count = curr_pos - self.last_pos;
        self.last_pos = curr_pos;
        // fill buffer with new samples
        for _ in 0..sample_count {
            let sample = self.gen_sample();
            self.ring_buffer.push_back(sample);
        }
    }

    pub fn new_frame(&mut self) {
        if self.ring_buffer.len() < SAMPLES {
            for _ in self.ring_buffer.len()..SAMPLES {
                self.ring_buffer.push_back(self.last_sample);
            }
        }
        self.last_pos = 0;
    }

    pub fn pop_buffer(&mut self) -> Option<SoundSample<i16>> {
        self.ring_buffer.pop_front()
    }
}
