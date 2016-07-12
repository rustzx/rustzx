use zx::sound::{SampleGenerator, SoundSample};

pub struct ZXBeeper {
    curr_bit: bool,
    next_bit: bool,
}

impl ZXBeeper {
    pub fn new() -> ZXBeeper {
        ZXBeeper {
            curr_bit: false,
            next_bit: false,
        }
    }

    pub fn change_bit(&mut self, value: bool) {
        self.curr_bit = value;
    }
}

impl SampleGenerator for ZXBeeper {
    fn gen_float_sample(&mut self) -> SoundSample<f64> {
        if self.curr_bit {
            1.0
        } else {
            -1.0
        }
    }
}
