use crate::{SoundChip, Stereo, Vtx};

/// Performs sound sample generation using provided vtx file
pub struct Player {
    vtx: Vtx,
    frame: usize,
    frame_sample: usize,
    stereo: bool,
    ay: ayumi::Ayumi,
    samples_per_frame: usize,
}

/// This trait is not meant to be implemented by the
/// client code and it is subject to change
pub trait PlayerSample {
    /// Constructs self from f64 sample
    fn from_f64(sample: f64) -> Self;
}

impl PlayerSample for f32 {
    fn from_f64(sample: f64) -> Self {
        sample as f32
    }
}

impl PlayerSample for i32 {
    fn from_f64(sample: f64) -> Self {
        (i32::MAX as f64 * sample).clamp(i32::MIN as f64, i32::MAX as f64) as i32
    }
}

impl PlayerSample for i16 {
    fn from_f64(sample: f64) -> Self {
        (i16::MAX as f64 * sample).clamp(i16::MIN as f64, i16::MAX as f64) as i16
    }
}

impl PlayerSample for i8 {
    fn from_f64(sample: f64) -> Self {
        (i8::MAX as f64 * sample).clamp(i8::MIN as f64, i8::MAX as f64) as i8
    }
}

impl Player {
    /// Constructs player instance from the given `vtx` file, `sample_rate` and `stereo`.
    /// `stereo` flag enables 2-channel sample generation.
    pub fn new(vtx: Vtx, sample_rate: usize, stereo: bool) -> Self {
        let chip_type = match vtx.chip {
            SoundChip::AY => ayumi::ChipType::AY,
            SoundChip::YM => ayumi::ChipType::YM,
        };

        let mut ay = ayumi::Ayumi::new(chip_type, vtx.frequency as f64, sample_rate as i32);

        if stereo {
            let (a_pan, b_pan, c_pan) = match vtx.stereo {
                Stereo::Mono => (0.5, 0.5, 0.5),
                Stereo::ABC => (0.0, 0.5, 1.0),
                Stereo::ACB => (0.0, 1.0, 0.5),
                Stereo::BAC => (0.5, 0.0, 1.0),
                Stereo::BCA => (1.0, 0.0, 0.5),
                Stereo::CAB => (0.5, 1.0, 0.0),
                Stereo::CBA => (1.0, 0.5, 0.0),
            };

            ay.tone(ayumi::ToneChannel::A).pan(a_pan, true);
            ay.tone(ayumi::ToneChannel::B).pan(b_pan, true);
            ay.tone(ayumi::ToneChannel::C).pan(c_pan, true);
        } else {
            ay.tone(ayumi::ToneChannel::A).pan(0.5, true);
            ay.tone(ayumi::ToneChannel::B).pan(0.5, true);
            ay.tone(ayumi::ToneChannel::C).pan(0.5, true);
        }

        let samples_per_frame = sample_rate / vtx.player_frequency as usize;

        Self {
            vtx,
            frame: 0,
            frame_sample: 0,
            stereo,
            ay,
            samples_per_frame,
        }
    }

    fn update_ay(&mut self) -> bool {
        if let Some(frame) = self.vtx.frame_registers(self.frame) {
            self.ay
                .tone(ayumi::ToneChannel::A)
                .period(u16::from_le_bytes([frame[0], frame[1] & 0x0f]) as i32);
            self.ay
                .tone(ayumi::ToneChannel::B)
                .period(u16::from_le_bytes([frame[2], frame[3] & 0x0f]) as i32);
            self.ay
                .tone(ayumi::ToneChannel::C)
                .period(u16::from_le_bytes([frame[4], frame[5] & 0x0f]) as i32);
            self.ay.noise().period((frame[6] & 0x1f) as i32);
            self.ay.tone(ayumi::ToneChannel::A).mixer(
                (frame[7] & 0x01) != 0,
                (frame[7] & 0x08) != 0,
                (frame[8] & 0x10) != 0,
            );
            self.ay.tone(ayumi::ToneChannel::B).mixer(
                (frame[7] & 0x02) != 0,
                (frame[7] & 0x10) != 0,
                (frame[9] & 0x10) != 0,
            );
            self.ay.tone(ayumi::ToneChannel::C).mixer(
                (frame[7] & 0x04) != 0,
                (frame[7] & 0x20) != 0,
                (frame[10] & 0x10) != 0,
            );
            self.ay.tone(ayumi::ToneChannel::A).volume(frame[8] & 0x0F);
            self.ay.tone(ayumi::ToneChannel::B).volume(frame[9] & 0x0F);
            self.ay.tone(ayumi::ToneChannel::C).volume(frame[10] & 0x0F);
            self.ay
                .envelope()
                .period(u16::from_le_bytes([frame[11], frame[12] & 0x0F]) as i32);

            if frame[13] != 0xFF {
                self.ay.envelope().shape(frame[13] & 0x0F);
            }
            return true;
        }

        false
    }

    /// Fills given `samples` slice with sound sample data.
    /// Returns quantity of samples which were filed in the buffer.
    /// When stereo mode is enabled, method returns overall samples count multiplied by
    /// thc channel count (2), so that regardless of the stereo mode there will be always
    /// `samples[..sample_count]` filled.
    pub fn play<S: PlayerSample>(&mut self, samples: &mut [S]) -> usize {
        let mut processed_samples = 0;
        if self.stereo {
            for sample in samples.chunks_exact_mut(2) {
                // On first frame sample - update ay state
                if self.frame_sample == 0 && !self.update_ay() {
                    return processed_samples * 2;
                }
                let ay_sample = self.ay.process().sample();
                sample[0] = S::from_f64(ay_sample.left);
                sample[1] = S::from_f64(ay_sample.right);

                processed_samples += 1;
                self.frame_sample += 1;
                if self.frame_sample == self.samples_per_frame {
                    self.frame_sample = 0;
                    self.frame += 1;
                }
            }
            processed_samples * 2
        } else {
            for sample in samples {
                if self.frame_sample == 0 && !self.update_ay() {
                    return processed_samples;
                }
                let ay_sample = self.ay.process().sample();
                *sample = S::from_f64(ay_sample.left);
                processed_samples += 1;
                self.frame_sample += 1;
                if self.frame_sample == self.samples_per_frame {
                    self.frame_sample = 0;
                    self.frame += 1;
                }
            }
            processed_samples
        }
    }

    /// Reset playback state
    pub fn rewind(&mut self) {
        self.frame = 0;
        self.frame_sample = 0;
        // Force reset envelope
        self.ay.envelope().shape(0);
    }

    /// Reset to start of the looped record
    pub fn rewind_loop(&mut self) {
        self.frame = self.vtx.loop_start_frame as usize;
        self.frame_sample = 0;
        self.ay.envelope().shape(0);
    }

    /// Sets frame position for playback
    pub fn set_frame(&mut self, frame: usize) -> bool {
        if frame >= self.vtx.frames_count() {
            return false;
        }
        self.frame = frame;
        self.frame_sample = 0;
        self.ay.envelope().shape(0);
        true
    }
}
