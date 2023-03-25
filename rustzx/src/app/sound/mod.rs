#[cfg(feature = "sound-cpal")]
mod sound_cpal;
mod sound_sdl;
use rustzx_core::zx::sound::sample::SoundSample;

#[cfg(feature = "sound-cpal")]
pub use sound_cpal::SoundCpal;
pub use sound_sdl::SoundSdl;

pub const CHANNEL_COUNT: usize = 2;
pub const DEFAULT_SAMPLE_RATE: usize = 44100;
pub const DEFAULT_LATENCY: usize = 512;

pub type ZXSample = SoundSample<f32>;

pub trait SoundDevice {
    /// Send new sample to the sound device
    fn send_sample(&mut self, sample: ZXSample);
    /// Return selected device sample rate
    fn sample_rate(&self) -> usize;
}

pub fn ringbuf_size_from_sample_rate(sample_rate: usize) -> usize {
    // Around 5 frames of buffering is available
    const RINGBUF_LENGTH_MS: usize = 100;
    (sample_rate as f32 * ( RINGBUF_LENGTH_MS as f32 / 1000f32)) as usize
}
