//! Module implements emulation of sound chip AY, Spectrum Beeper and Mixer
pub mod ay;
pub mod beeper;
pub mod mixer;
pub mod sample;

use crate::zx::constants::FPS;

// TODO: Make sample rate configurable
pub const SAMPLE_RATE: usize = 44100;
/// samples per frame
pub const SAMPLES: usize = SAMPLE_RATE / FPS;
pub const CHANNELS: usize = 2;

/// Returns, which must be already processed at this time
pub fn samples_from_time(time: f64) -> usize {
    if time >= 1.0 {
        SAMPLES
    } else if time <= 0.0 {
        0
    } else {
        (SAMPLES as f64 * time) as usize
    }
}
