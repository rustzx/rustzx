//! Real Audio SDL backend
use super::{SoundDevice, ZXSample};
use crate::{
    backends::SDL_CONTEXT,
    settings::RustzxSettings,
    zx::sound::{CHANNELS, SAMPLE_RATE},
};
use sdl2::audio::{AudioCallback, AudioDevice, AudioSpecDesired};
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};

/// Struct which used in SDL audio callback
struct SdlCallback {
    samples: Receiver<ZXSample>,
}

impl AudioCallback for SdlCallback {
    type Channel = f32;

    /// main callback function
    fn callback(&mut self, out: &mut [f32]) {
        for chunk in out.chunks_mut(CHANNELS) {
            // recieve samples from channel
            if let Some(sample) = self.samples.recv().ok() {
                chunk[0] = sample.left;
                chunk[1] = sample.right;
            }
        }
    }
}

/// Represents SDL audio backend
pub struct SoundSdl {
    sender: SyncSender<ZXSample>,
    device: AudioDevice<SdlCallback>,
}

impl SoundSdl {
    /// constructs sound backend from settings
    pub fn new(settings: &RustzxSettings) -> SoundSdl {
        // init backend
        let mut audio_subsystem = None;
        SDL_CONTEXT.with(|sdl| {
            audio_subsystem = sdl.borrow_mut().audio().ok();
        });
        if let Some(audio) = audio_subsystem {
            // prepare specs
            let desired_spec = AudioSpecDesired {
                freq: Some(SAMPLE_RATE as i32),
                channels: Some(CHANNELS as u8),
                samples: Some(settings.latency as u16),
            };
            let (tx, rx) = sync_channel(settings.latency as usize);
            let device_handle = audio
                .open_playback(None, &desired_spec, |_| SdlCallback { samples: rx })
                .expect("[ERROR Sdl audio device error, try --nosound]");
            // run
            device_handle.resume();
            // save device and cahnnel handles
            SoundSdl {
                sender: tx,
                device: device_handle,
            }
        } else {
            panic!("[ERROR] Sdl audio error, try --nosound");
        }
    }
}

impl SoundDevice for SoundSdl {
    fn send_sample(&mut self, sample: ZXSample) {
        self.sender.send(sample).unwrap();
    }
}
