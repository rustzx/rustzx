//! Module implements zx spectrum audio devices mixer
use crate::zx::{
    constants::FPS,
    sound::{
        beeper::ZXBeeper,
        sample::{SampleGenerator, SoundSample},
    },
};

// TODO(#117): Implement DC filtering for sound mixing

#[cfg(feature = "ay")]
use crate::zx::sound::ay::{ZXAYMode, ZXAyChip};

use alloc::collections::VecDeque;

/// Main sound mixer.
pub(crate) struct ZXMixer {
    /// direct access to beeper device
    pub beeper: ZXBeeper,
    /// direct access to AY device
    #[cfg(feature = "ay")]
    pub ay: ZXAyChip,
    ring_buffer: VecDeque<SoundSample<f32>>,
    last_pos: usize,
    last_sample: SoundSample<f32>,
    master_volume: f64,
    #[cfg(feature = "ay")]
    pub use_ay: bool,
    use_beeper: bool,
    sample_rate: usize,
}

impl ZXMixer {
    /// Constructs new Mixer structure
    /// # Arguments
    /// - `use_beeper` - process beeper or not
    /// - `use_ay` - process ay chip or not
    pub fn new(
        use_beeper: bool,
        #[cfg(feature = "ay")] use_ay: bool,
        #[cfg(feature = "ay")] ay_mode: ZXAYMode,
        sample_rate: usize,
    ) -> ZXMixer {
        ZXMixer {
            beeper: ZXBeeper::default(),
            #[cfg(feature = "ay")]
            ay: ZXAyChip::new(sample_rate, ay_mode),
            ring_buffer: VecDeque::with_capacity(sample_rate),
            last_pos: 0,
            last_sample: SoundSample::new(0.0, 0.0),
            master_volume: 0.5,
            #[cfg(feature = "ay")]
            use_ay,
            use_beeper,
            sample_rate,
        }
    }

    /// changes volume
    /// # Arguments
    /// - `volume` - value in range 0..1
    pub fn volume(&mut self, volume: f64) {
        self.master_volume = volume;
    }

    /// Updates internal buffer of mixer and fills it with new samples
    pub fn process(&mut self, current_time: f64) {
        // buffer overflow
        if self.ring_buffer.len() >= self.samples_per_frame() {
            return;
        }
        // so at this moment we need to get new samples from devices
        let curr_pos = self.sample_count_for_frame_fraction(current_time);
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

    /// fills buffer to eng on new frame
    pub fn new_frame(&mut self) {
        if self.ring_buffer.len() < self.samples_per_frame() {
            for _ in self.ring_buffer.len()..self.samples_per_frame() {
                self.ring_buffer.push_back(self.last_sample);
            }
        }
        self.last_pos = 0;
    }

    pub fn pop(&mut self) -> Option<SoundSample<f32>> {
        self.ring_buffer.pop_front()
    }

    fn gen_sample(&mut self) -> SoundSample<f32> {
        let mut master_float = if self.use_beeper {
            self.beeper.gen_sample()
        } else {
            SoundSample::new(0.0, 0.0)
        };
        #[cfg(feature = "ay")]
        if self.use_ay {
            master_float.mix(&self.ay.gen_sample());
        }
        let master = master_float.mul_eq(self.master_volume).into_f32();
        self.last_sample = master;
        master
    }

    fn samples_per_frame(&self) -> usize {
        self.sample_rate / FPS
    }

    fn sample_count_for_frame_fraction(&self, fraction: f64) -> usize {
        if fraction >= 1f64 {
            return self.samples_per_frame();
        }
        (self.samples_per_frame() as f64 * fraction) as usize
    }
}
