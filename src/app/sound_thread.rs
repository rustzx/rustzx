//! TODO: add error handling
use std::i16;
use std::sync::mpsc::*;
use std::cell::Cell;
use zx::constants::SAMPLE_RATE;

use portaudio as pa;

// Mono
const CHANNELS: i32 = 1;
// get 256 samples per one callback
const FRAMES_PER_BUFFER: u32 = 256;
// 64 K buffer for sound
const BUFFER_SIZE: usize = 1024;

type SpeakerStream<'a> = pa::stream::Stream<'a, pa::stream::NonBlocking,pa::stream::Output<i16>>;

lazy_static! {
    pub static ref PA_STATIC: pa::PortAudio = {
        pa::PortAudio::new().unwrap()
    };
}

pub struct SoundThread<'a> {
    channel: Option<SyncSender<i16>>,
    stream: Option<SpeakerStream<'a>>,
}

impl<'a> SoundThread<'a> {
    pub fn new() -> SoundThread<'a> {
        SoundThread {
            channel: None,
            stream: None,
        }
    }
    pub fn run_sound_thread(&mut self) {
        // settings for stream
        let mut settings = PA_STATIC.default_output_stream_settings::<i16>(CHANNELS,
                                                                           SAMPLE_RATE as f64,
                                                                           FRAMES_PER_BUFFER)
                                                                           .unwrap();
        settings.flags = pa::stream_flags::CLIP_OFF;
        // open channel for messages
        let (tx, rx) = sync_channel(BUFFER_SIZE);
        let last_state = Cell::new(i16::min_value());
        // set callback
        let callback = move |pa::OutputStreamCallbackArgs { buffer, frames, .. }| {
            // index to buffer
            let mut n = 0;
            // last value of "speaker bit"
            let mut last = last_state.get();
            loop {
                // if provided more samples than needed then break loop
                if n >= frames {
                    break;
                }
                // if we have sample then write it to the buffer
                if let Some(sample) = rx.try_recv().ok() {
                    last = sample;
                    buffer[n] = sample;
                } else {
                    break;
                }
                // write successefull, increment index
                n += 1;
            }
            last_state.set(last);
            // fill to end
            if n < frames {
                for k in n..frames {
                    buffer[k] = last;
                }
            }
            // continue streaming
            pa::Continue
        };
        // save channel and stream to handle
        self.channel = Some(tx);
        let mut stream = PA_STATIC.open_non_blocking_stream(settings, callback).unwrap();
        stream.start().unwrap();
        self.stream = Some(stream);
    }
    pub fn send(& mut self, value: i16) {
        if let Some(ref channel) = self.channel {
            channel.send(value).unwrap();
        };
    }
}
