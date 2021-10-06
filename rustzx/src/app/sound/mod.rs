mod sound_cpal;
mod sound_sdl;
use rustzx_core::zx::sound::sample::SoundSample;

pub use sound_cpal::SoundCpal;
pub use sound_sdl::SoundSdl;

pub const CHANNEL_COUNT: usize = 2;
pub const DEFAULT_SAMPLE_RATE: usize = 44100;

pub type ZXSample = SoundSample<f32>;

pub trait SoundDevice {
    /// Send new sample to the sound device
    fn send_sample(&mut self, sample: ZXSample);
    /// Return selected device sample rate
    fn sample_rate(&self) -> usize;
}
