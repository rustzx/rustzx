mod sound_sdl;
pub use self::sound_sdl::SoundSdl;
use zx::sound::sample::SoundSample;

pub type ZXSample = SoundSample<f32>;
pub trait SoundDevice {
    // blocking function to send new sample
    fn send_sample(&mut self, sample: ZXSample);
}
