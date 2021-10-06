use crate::zx::sound::sample::{SampleGenerator, SoundSample};

/// Simple beeper implementation
#[derive(Default)]
pub(crate) struct ZXBeeper {
    mic: bool,
    ear: bool,
}

impl ZXBeeper {
    /// Changes next beeper bit
    pub fn change_state(&mut self, ear: bool, mic: bool) {
        self.ear = ear;
        self.mic = mic;
    }
}

impl SampleGenerator<f64> for ZXBeeper {
    fn gen_sample(&mut self) -> SoundSample<f64> {
        // - Beeper intentionally made produce only positive half-wave 0..0.5
        // range instead of -0.25..0.25) because of current emulator lack of
        // dc filtering.
        // - Beeper only produces a quater of available sample
        // range because relatively to AY chip, square wave of a beeper is
        // too loud

        const EAR_SAMPLE_FACTOR: f64 = 0.5;
        const MIC_SAMPLE_FACTOR: f64 = EAR_SAMPLE_FACTOR / 5.0;

        let mut sample = 0.0;
        if self.ear {
            sample += EAR_SAMPLE_FACTOR;
        }
        if self.mic {
            sample += MIC_SAMPLE_FACTOR;
        }

        SoundSample::new(sample, sample)
    }
}
