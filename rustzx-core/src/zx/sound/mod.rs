//! Module implements emulation of sound chip AY, Spectrum Beeper and Mixer
#[cfg(feature = "ay")]
pub mod ay;
pub mod sample;

pub(crate) mod beeper;
pub(crate) mod mixer;

use crate::zx::constants::FPS;

// TODO(#62): Make sample rate configurable
pub const SAMPLE_RATE: usize = 44100;
/// samples per frame
pub const SAMPLES: usize = SAMPLE_RATE / FPS;
// TODO: rename to samples per frame
pub const CHANNELS: usize = 2;

/// Returns, which must be already processed at this time
pub(crate) fn samples_from_time(time: f64) -> usize {
    if time >= 1.0 {
        SAMPLES
    } else if time <= 0.0 {
        0
    } else {
        (SAMPLES as f64 * time) as usize
    }
}
