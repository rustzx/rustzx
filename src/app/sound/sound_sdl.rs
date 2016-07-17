use super::{SoundDevice, ZXSample};
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use sdl2;
use sdl2::audio::{AudioCallback, AudioSpecDesired, AudioDevice};
use zx::sound::{CHANNELS, SAMPLE_RATE};

const BUFFER_SIZE: usize = 512;
const FRAME_SIZE: u16 = 256;

struct SdlCallback {
    samples: Receiver<ZXSample>
}

impl AudioCallback for SdlCallback {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        for chunk in out.chunks_mut(CHANNELS) {
            if let Some(sample) = self.samples.recv().ok() {
                chunk[0] = sample.left;
                chunk[1] = sample.right;
            }
        }
    }
}



pub struct SoundSdl {
    sender: SyncSender<ZXSample>,
    device: AudioDevice<SdlCallback>,
}

impl SoundSdl {
    pub fn new() -> SoundSdl {
        let sdl_context = sdl2::init().expect("[ERROR] Sdl init error, try --nosound");
        let audio_subsystem = sdl_context.audio().expect("[ERROR] Sdl audio error, try --nosound");
        let desired_spec = AudioSpecDesired {
            freq: Some(SAMPLE_RATE as i32),
            channels: Some(CHANNELS as u8),
            samples: Some(FRAME_SIZE),
        };
        let (tx, rx) = sync_channel(BUFFER_SIZE);
        let device_handle = audio_subsystem.open_playback(None, &desired_spec, |spec| {
            // Show obtained AudioSpec
            println!("{:?}", spec);

            SdlCallback {
                samples: rx,
            }
        }).expect("[ERROR Sdl audio device error, try --nosound]");
        device_handle.resume();
        SoundSdl {
            sender: tx,
            device: device_handle,
        }
    }
}
impl SoundDevice for SoundSdl {
    fn send_sample(&mut self, sample: ZXSample) {
        self.sender.send(sample).unwrap();
    }
}
