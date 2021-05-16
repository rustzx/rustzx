use crate::{SoundChip, Stereo, Vtx};
use aym::{AySample, AymBackend};
use num_traits::Num;

pub type PrecisePlayer = Player<aym::AymPrecise>;

/// Performs sound sample generation using provided vtx file
pub struct Player<AY: AymBackend> {
    vtx: Vtx,
    frame: usize,
    frame_sample: usize,
    stereo: bool,
    ay: AY,
    samples_per_frame: usize,
}

/// This trait is not meant to be implemented by the
/// client code and it is subject to change
pub trait PlayerSample: Num {
    /// Constructs self from f64 sample
    fn from_aym_sample(sample: impl aym::AySample) -> Self;
}

impl PlayerSample for i8 {
    fn from_aym_sample(sample: impl AySample) -> Self {
        sample.to_i8()
    }
}

impl PlayerSample for i16 {
    fn from_aym_sample(sample: impl AySample) -> Self {
        sample.to_i16()
    }
}

impl PlayerSample for i32 {
    fn from_aym_sample(sample: impl AySample) -> Self {
        sample.to_i32()
    }
}

impl PlayerSample for f32 {
    fn from_aym_sample(sample: impl AySample) -> Self {
        sample.to_f32()
    }
}

impl PlayerSample for f64 {
    fn from_aym_sample(sample: impl AySample) -> Self {
        sample.to_f64()
    }
}

impl<AY: AymBackend> Player<AY> {
    /// Constructs player instance from the given `vtx` file, `sample_rate` and `stereo`.
    /// `stereo` flag enables 2-channel sample generation.
    pub fn new(vtx: Vtx, sample_rate: usize, stereo: bool) -> Self {
        let chip_type = match vtx.chip {
            SoundChip::AY => aym::SoundChip::AY,
            SoundChip::YM => aym::SoundChip::YM,
        };

        let mode = if stereo {
            match vtx.stereo {
                Stereo::Mono => aym::AyMode::Mono,
                Stereo::ABC => aym::AyMode::ABC,
                Stereo::ACB => aym::AyMode::ACB,
                Stereo::BAC => aym::AyMode::BAC,
                Stereo::BCA => aym::AyMode::BCA,
                Stereo::CAB => aym::AyMode::CAB,
                Stereo::CBA => aym::AyMode::CBA,
            }
        } else {
            aym::AyMode::Mono
        };

        let ay = AY::new(chip_type, mode, vtx.frequency as usize, sample_rate);

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
            for (idx, value) in frame.iter().copied().enumerate() {
                if idx == 13 && value == 0xFF {
                    continue;
                }
                self.ay.write_register(idx as u8, value);
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
                let ay_sample = self.ay.next_sample();
                sample[0] = S::from_aym_sample(ay_sample.left);
                sample[1] = S::from_aym_sample(ay_sample.right);

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
                let ay_sample = self.ay.next_sample();
                *sample = S::from_aym_sample(ay_sample.left);
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
        self.ay.write_register(13, 0);
    }

    /// Reset to start of the looped record
    pub fn rewind_loop(&mut self) {
        self.frame = self.vtx.loop_start_frame as usize;
        self.frame_sample = 0;
        self.ay.write_register(13, 0);
    }

    /// Sets frame position for playback
    pub fn set_frame(&mut self, frame: usize) -> bool {
        if frame >= self.vtx.frames_count() {
            return false;
        }
        self.frame = frame;
        self.frame_sample = 0;
        self.ay.write_register(13, 0);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use expect_test::expect;
    use sha2::{Digest, Sha256};

    fn generate_samples_hash_for_vtx<B: AymBackend>(data: &[u8]) -> String {
        const SAMPLE_RATE: usize = 44100;

        let vtx = Vtx::load(std::io::Cursor::new(data)).unwrap();
        let buffer_size = SAMPLE_RATE * 15; // 15 seconds will be enough for validation
        let mut buffer = vec![0i16; buffer_size];
        let mut player = PrecisePlayer::new(vtx, SAMPLE_RATE, true);
        let actual_length = player.play(&mut buffer);
        buffer.truncate(actual_length);

        let mut hashable_buffer = Vec::with_capacity(buffer.len() * 2);
        buffer.into_iter().for_each(|sample| {
            hashable_buffer.extend_from_slice(&sample.to_ne_bytes());
        });

        let mut hasher = Sha256::default();
        hasher.update(&hashable_buffer);
        let frame_data_fingerprint = hasher.finalize();
        format!("{:x}", frame_data_fingerprint)
    }

    #[test]
    fn precise_player_wroks_normally() {
        // This test also doubles as test for `aym` crate with AymPrecise backend selected
        expect![[r#"65127ad0e493b23b43e838b6281d7f8d07a47549efd0c9d1eda3d3ccaf0b7e5c"#]].assert_eq(
            &generate_samples_hash_for_vtx::<aym::AymPrecise>(include_bytes!("test/csoon.vtx")),
        )
    }
}
