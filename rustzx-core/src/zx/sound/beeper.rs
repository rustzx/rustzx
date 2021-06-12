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
