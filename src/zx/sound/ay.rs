use utils::make_word;
use zx::constants::SAMPLE_RATE;
use zx::sound::*;
use ayumi::*;

// AY chip runs on the same frequency on 128K, 2+, 3+
const AY_FREQ: f64 = 1773400.0;

pub struct ZXAyChip {
    ay: Ayumi,
    current_reg: usize,
    regs: [u8; 16],
}

impl ZXAyChip {
    /// Constructs new AY Chip
    pub fn new() -> ZXAyChip {
        // configure ayumi
        let mut ayumi = Ayumi::new(ChipType::AY, AY_FREQ, SAMPLE_RATE as i32);
        ayumi.tone(ToneChannel::A).pan(0.5, true);
        ayumi.tone(ToneChannel::B).pan(0.5, true);
        ayumi.tone(ToneChannel::C).pan(0.5, true);
        ZXAyChip {
            ay: ayumi,
            current_reg: 0,
            regs: [0; 16],
        }
    }

    /// Selects active AY register to write
    pub fn select_reg(&mut self, reg: u8) {
        // AY chip have only 16 regs [0...15]
        self.current_reg = (reg & 0x0F) as usize;
    }

    /// Tries to write some data to AY registers
    pub fn write(&mut self, data: u8) {
        let reg = self.current_reg;
        self.regs[reg] = data;
        match self.current_reg {
            // Channel A tone period
            0 ... 1 => {
                let word = make_word(self.regs[1] & 0x0F, self.regs[0]);
                self.ay.tone(ToneChannel::A).period(word as i32);
            }
            // Channel B tone period
            2 ... 3 => {
                let word = make_word(self.regs[3] & 0x0F, self.regs[2]);
                self.ay.tone(ToneChannel::B).period(word as i32);
            }
            // Channel C tone period
            4 ... 5 => {
                let word = make_word(self.regs[5] & 0x0F, self.regs[4]);
                self.ay.tone(ToneChannel::C).period(word as i32);
            }
            // Noise period
            6 => {
                self.ay.noise().period((self.regs[6] & 0x1F) as i32);
            }
            // Mixer Controls
            7 ... 10 => {
                self.ay.tone(ToneChannel::A)
                       .mixer((self.regs[7] & 0x01) != 0,
                              (self.regs[7] & 0x08) != 0,
                              (self.regs[8] & 0x10) != 0);
                self.ay.tone(ToneChannel::B)
                       .mixer((self.regs[7] & 0x02) != 0,
                              (self.regs[7] & 0x10) != 0,
                              (self.regs[9] & 0x10) != 0);
                self.ay.tone(ToneChannel::C)
                       .mixer((self.regs[7] & 0x04) != 0,
                              (self.regs[7] & 0x20) != 0,
                              (self.regs[10] & 0x10) != 0);
                if self.current_reg > 7 {
                    self.ay.tone(ToneChannel::A)
                           .volume(self.regs[8] & 0x0F);
                    self.ay.tone(ToneChannel::B)
                           .volume(self.regs[9] & 0x0F);
                    self.ay.tone(ToneChannel::C)
                           .volume(self.regs[10] & 0x0F);
                }
            }
            // Envelope period
            11 ... 12 => {
                let word = make_word(self.regs[12] & 0x0F, self.regs[11]);
                self.ay.envelope().period(word as i32);
            }
            // Envelope Shape
            13 => {
                self.ay.envelope().shape(self.regs[13] & 0x0F);
            }
            // Just don't handle IO ports
            _ => {}
        }
        // find out what we need to do with value
    }

    pub fn read(&self) -> u8 {
        self.regs[self.current_reg]
    }
}

impl SampleGenerator for ZXAyChip {
    fn gen_float_sample(&mut self) -> SoundSample<f64> {
        // pan of channels is equal we'll get single channel as a sample value
        let sample = self.ay.process().remove_dc().sample().right;
        sample
    }
}
