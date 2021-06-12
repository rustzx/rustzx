use crate::zx::sound::sample::{SampleGenerator, SoundSample};

/// Simple beeper implementation
#[derive(Default)]
pub(crate) struct ZXBeeper {
    curr_bit: bool,
    next_bit: bool,
}

impl ZXBeeper {
    /// Changes next beeper bit
    pub fn change_bit(&mut self, value: bool) {
        self.curr_bit = self.next_bit;
        self.next_bit = value;
    }
}

impl SampleGenerator<f64> for ZXBeeper {
    fn gen_sample(&mut self) -> SoundSample<f64> {
        if self.curr_bit {
            SoundSample::new(0.50, 0.50)
        } else {
            SoundSample::new(0.0, 0.0)
        }
    }
}
