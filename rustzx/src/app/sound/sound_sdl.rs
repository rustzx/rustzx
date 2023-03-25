use crate::{
    app::{
        settings::Settings,
        sound::{SoundDevice, ZXSample, CHANNEL_COUNT, DEFAULT_LATENCY, DEFAULT_SAMPLE_RATE, ringbuf_size_from_sample_rate},
    },
    backends::SDL_CONTEXT,
};
use sdl2::audio::{AudioCallback, AudioDevice, AudioSpecDesired};
use std::sync::Arc;

/// Struct which used in SDL audio callback
struct SdlCallback {
    samples: ringbuf::Consumer<ZXSample, Arc<ringbuf::HeapRb::<ZXSample>>>,
}

impl AudioCallback for SdlCallback {
    type Channel = f32;

    /// main callback function
    fn callback(&mut self, out: &mut [f32]) {
        for chunk in out.chunks_mut(CHANNEL_COUNT) {
            // recieve samples from channel
            if let Some(sample) = self.samples.pop() {
                chunk[0] = sample.left;
                chunk[1] = sample.right;
            } else {
                chunk[0] = 0f32;
                chunk[1] = 0f32;
            }
        }
    }
}

/// Represents SDL audio backend
pub struct SoundSdl {
    sender: ringbuf::Producer<ZXSample, Arc<ringbuf::HeapRb::<ZXSample>>>,
    sample_rate: usize,
    _device: AudioDevice<SdlCallback>, // Should be alive until Drop invocation
}

impl SoundSdl {
    /// constructs sound backend from settings
    pub fn new(settings: &Settings) -> anyhow::Result<SoundSdl> {
        let mut audio_subsystem = None;
        SDL_CONTEXT.with(|sdl| {
            audio_subsystem = sdl.borrow_mut().audio().ok();
        });
        let audio = audio_subsystem
            .ok_or_else(|| anyhow::anyhow!("Failed to initialize SDL audio backend"))?;

        // Basically, SDL shits its pants if desired sound sample rate is not specified
        let sample_rate = settings.sound_sample_rate.unwrap_or(DEFAULT_SAMPLE_RATE);

        // SDL sets awfully big latency by default
        let latency = settings.sound_latency.unwrap_or(DEFAULT_LATENCY);

        let desired_spec = AudioSpecDesired {
            freq: Some(sample_rate as i32),
            channels: Some(CHANNEL_COUNT as u8),
            samples: Some(latency as u16),
        };
        let ringbuf_size = ringbuf_size_from_sample_rate(sample_rate);
        let ringbuf = ringbuf::HeapRb::<ZXSample>::new(ringbuf_size);
        let (tx, rx) = ringbuf.split();

        let device_handle = audio
            .open_playback(None, &desired_spec, |_| SdlCallback { samples: rx })
            .map_err(|e| anyhow::anyhow!("Failed to start SDL sound stream: {}", e))?;
        device_handle.resume();

        Ok(SoundSdl {
            sender: tx,
            sample_rate,
            _device: device_handle,
        })
    }
}

impl SoundDevice for SoundSdl {
    fn send_sample(&mut self, sample: ZXSample) {
        // Ignore sample push errors
        let _ = self.sender.push(sample);
    }

    fn sample_rate(&self) -> usize {
        self.sample_rate
    }
}
