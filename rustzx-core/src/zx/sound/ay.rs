use crate::zx::sound::sample::{SampleGenerator, SoundSample};
use aym::{AyMode, AymBackend, AymPrecise, SoundChip};

/// AY chip runs on the same frequency on 128K, 2+, 3+
const AY_FREQ: usize = 1773400;

/// AY output mode
#[derive(Clone, Copy)]
#[allow(clippy::upper_case_acronyms)]
pub enum ZXAYMode {
    Mono,
    ABC,
    ACB,
}

pub(crate) struct ZXAyChip {
    ay: AymPrecise,
    current_reg: usize,
    regs: [u8; 16],
}

impl ZXAyChip {
    pub fn new(sample_rate: usize, mode: ZXAYMode) -> ZXAyChip {
        let mode = match mode {
            ZXAYMode::Mono => AyMode::Mono,
            ZXAYMode::ABC => AyMode::ABC,
            ZXAYMode::ACB => AyMode::ACB,
        };

        Self {
            ay: AymBackend::new(SoundChip::AY, mode, AY_FREQ, sample_rate),
            current_reg: 0,
            regs: [0; 16],
        }
    }

    pub fn select_reg(&mut self, reg: u8) {
        // AY chip have only 16 regs [0..=15]
        self.current_reg = (reg & 0x0F) as usize;
    }

    pub fn write(&mut self, data: u8) {
        let reg = self.current_reg;
        self.regs[reg] = data;
        self.ay.write_register(reg as u8, data);
    }

    pub fn read(&self) -> u8 {
        self.regs[self.current_reg]
    }
}

impl SampleGenerator<f64> for ZXAyChip {
    fn gen_sample(&mut self) -> SoundSample<f64> {
        let sample = self.ay.next_sample();

        // Place -1..1 sample in 0..1 range
        let left = (sample.left + 1f64) / (2f64);
        let right = (sample.right + 1f64) / (2f64);

        SoundSample::new(left, right)
    }
}
