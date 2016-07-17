//! portaudio-based sound rendering module
//! PortAudio lib inits lazily, so if any error happens,
//! You can use --nosound flag
//! ### abbr
//! PA = `PortAudio`
use std::i16;
use std::sync::mpsc::*;
use std::cell::Cell;
use zx::sound::{CHANNELS, SAMPLE_RATE};
use zx::sound::sample::SoundSample;

use portaudio as pa;

/// 256 samples per one callback
const FRAMES_PER_BUFFER: u32 = 256;
/// 1K samples buffer for sound
const BUFFER_SIZE: usize = 1024;

/// type that describes PortAudio outout stream
type SpeakerStream<'a> = pa::stream::Stream<'a, pa::stream::NonBlocking, pa::stream::Output<i16>>;

// init PA
lazy_static! {
    pub static ref PA_STATIC: pa::PortAudio = {
        pa::PortAudio::new().ok().expect("[ERROR] PortAudio initialization error, try to use\
                                          --nosound option")
    };
}

pub struct SoundThread<'a> {
    channel: Option<SyncSender<SoundSample<i16>>>,
    stream: Option<SpeakerStream<'a>>,
}

impl<'a> SoundThread<'a> {
    /// Constructs new thread
    pub fn new() -> SoundThread<'a> {
        // construct new sound thread, everything is closed before running sound thread
        SoundThread {
            channel: None,
            stream: None,
        }
    }
    /// Runs sound thread
    pub fn run_sound_thread(&mut self) {
        // settings for stream
        let settings = PA_STATIC.default_output_stream_settings::<i16>(CHANNELS as i32,
                                                                       SAMPLE_RATE as f64,
                                                                       FRAMES_PER_BUFFER);
        let mut settings = settings.ok()
                                   .expect("[ERROR] PortAudio output stream creation error,try \
                                            to use --nosound option");
        settings.flags = pa::stream_flags::CLIP_OFF;
        // open channel for messages between main and sound thread
        let (tx, rx) = sync_channel(BUFFER_SIZE);
        // cell for storeing last state of sound playback
        let last_state = Cell::new(SoundSample::new(0, 0));
        // set callback
        let callback = move |pa::OutputStreamCallbackArgs { buffer, frames, .. }| {
            // index to buffer
            let mut n = 0;
            // last value of "speaker bit"
            let mut last = last_state.get();
            for _ in 0..frames {
                if let Some(sample) = rx.try_recv().ok() {
                    last = sample;
                    buffer[n] = sample.left;
                    buffer[n + 1] = sample.right;
                } else {
                    buffer[n] = last.left;
                    buffer[n + 1] = last.right;
                }
                n += 2;
            }
            last_state.set(last);
            pa::Continue
        };
        // save channel and stream to handle
        self.channel = Some(tx);
        let mut stream = PA_STATIC.open_non_blocking_stream(settings, callback)
                                  .ok()
                                  .expect("[ERROR] PortAudio stream oppening error,\
                                           try to use --nosound option");
        stream.start()
              .ok()
              .expect("[ERROR] PortAudio throwed error on stream start,try to use --nosound \
                       option");
        // store reference to stream
        self.stream = Some(stream);
    }
    pub fn send(&mut self, value: SoundSample<i16>) {
        if let Some(ref channel) = self.channel {
            channel.send(value)
                   .ok()
                   .expect("[ERROR] Sound sample sending failed, trye to use--nosound option");
        };
    }
}
