//! Module implements emulation of sound chip AY and
//! Spectrum Beeper
//!
//! Mixer contains both devices and have methods for controlling them
//! Both beeper and AY can be accessed as pub fields
pub mod beeper;
pub mod ay;
pub mod mixer;
//pub use self::ay::*;


use zx::constants::{SAMPLE_RATE, FPS};
/// samples per frame
pub const SAMPLES: usize = SAMPLE_RATE / FPS;

/// internal sample type
pub type SoundSample<T> = T;

pub trait SampleGenerator {
    fn gen_float_sample(&mut self) -> SoundSample<f64>;
}

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
