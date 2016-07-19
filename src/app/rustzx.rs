//! Main application class module
//! Handles all platform-related, hardware-related stuff
//! and command-line interface

use std::thread;
use std::time::Duration;
use time;
use app::sound::*;
use app::video::*;
use app::events::*;
use zx::constants::*;
use settings::RustzxSettings;
use emulator::*;


/// max 100 ms interval in `max frames` speed mode
const MAX_FRAME_TIME_NS: u64 = 100 * 1000000;

/// converts nanoseconds  to miliseconds
fn ns_to_ms(ns: u64) -> f64 {
    ns as f64 / 1_000_000f64
}

/// converts miliseconds to nanoseconds
fn ms_to_ns(s: f64) -> u64 {
    (s * 1_000_000_f64) as u64
}

/// returns frame length from given `fps`
fn frame_length_ns(fps: usize) -> u64 {
    ms_to_ns(1000 as f64 / fps as f64)
}

// TODO: FIX! MAKE BULDER
/// Application instance type
pub struct RustzxApp {
    /// main emulator object
    emulator: Emulator,
    /// Sound rendering in a separate thread
    snd: Option<Box<SoundDevice>>,
    video: Box<VideoDevice>,
    events: Box<EventDevice>,
    tex_border: TextureInfo,
    tex_canvas: TextureInfo,
    settings: RustzxSettings,
}

impl RustzxApp {
    /// Starts application itself
    pub fn from_config(settings: RustzxSettings) -> RustzxApp {
        let snd: Option<Box<SoundDevice>> = if settings.sound_enabled {
            Some(Box::new(SoundSdl::new(&settings)))
        } else {
            None
        };
        let mut video = Box::new(VideoSdl::new(&settings));
        let tex_border = video.gen_texture(SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32);
        let tex_canvas = video.gen_texture(CANVAS_WIDTH as u32, CANVAS_HEIGHT as u32);
        RustzxApp {
            emulator: Emulator::new(&settings),
            snd: snd,
            video: video,
            events: Box::new(EventsSdl::new(&settings)),
            tex_border: tex_border,
            tex_canvas: tex_canvas,
            settings: settings,
        }
    }

    pub fn start(&mut self) {
        let mut debug = false;
        let scale = self.settings.scale as u32;
        'emulator: loop {
            let frame_target_dt_ns = frame_length_ns(FPS);
            // absolute start time
            let frame_start_ns = time::precise_time_ns();
            // Emulate all requested frames
            let cpu_dt_ns = self.emulator.emulate_frames(MAX_FRAME_TIME_NS);
            // if sound enabled sound ganeration allowed then move samples to sound thread
            if let Some(ref mut snd) = self.snd {
                // if can be turned off even on speed change, so check it everytime
                if self.emulator.have_sound() {
                    loop {
                        if let Some(sample) = self.emulator.controller.mixer.pop() {
                            snd.send_sample(sample);
                        } else {
                            break;
                        }
                    }
                }
            }
            // load new textures to sdl
            self.video.update_texture(self.tex_border, self.emulator.controller.border.texture());
            self.video.update_texture(self.tex_canvas, self.emulator.controller.canvas.texture());
            // rendering block
            self.video.begin();
            self.video.draw_texture_2d(self.tex_border,
                                       Some(Rect::new(0,
                                                      0,
                                                      SCREEN_WIDTH as u32 * scale,
                                                      SCREEN_HEIGHT as u32 * scale)));
            self.video.draw_texture_2d(self.tex_canvas,
                                       Some(Rect::new(CANVAS_X as i32 * scale as i32,
                                                      CANVAS_Y as i32 * scale as i32,
                                                      CANVAS_WIDTH as u32 * scale,
                                                      CANVAS_HEIGHT as u32 * scale)));
            self.video.end();
            // check all events
            if let Some(event) = self.events.pop_event() {
                match event {
                    Event::Exit => {
                        break 'emulator;
                    }
                    Event::GameKey(key, state) => {
                        self.emulator.controller.send_key(key, state);
                    }
                    Event::SwitchDebug => {
                        debug = !debug;
                        if !debug {
                            self.video.set_title(&format!("RustZX v{}",
                                                        env!("CARGO_PKG_VERSION")));
                        }
                    }
                    Event::ChangeSpeed(speed) => {
                        self.emulator.set_speed(speed);
                    }
                    Event::Kempston(key, state) => {
                        if let Some(ref mut joy) = self.emulator.controller.kempston {
                            joy.key(key, state);
                        }
                    }
                    Event::InsertTape => {
                        self.emulator.controller.tape.play()
                    }
                    Event::StopTape => {
                        self.emulator.controller.tape.stop()
                    }
                }
            }
            // how long emulation iteration was
            let emulation_dt_ns = time::precise_time_ns() - frame_start_ns;
            if emulation_dt_ns < frame_target_dt_ns {
                let wait_koef = if self.emulator.have_sound() {
                    0.9
                } else {
                    1.0
                };
                // sleep untill frame sync
                thread::sleep(Duration::new(
                    0, ((frame_target_dt_ns - emulation_dt_ns) as f64 * wait_koef) as u32));
            };
            // get exceed clocks and use them on next iteration
            let frame_dt_ns = time::precise_time_ns() - frame_start_ns;
            // change window header
            if debug {
                self.video.set_title(&format!("CPU: {:7.3}ms; FRAME:{:7.3}ms",
                                               ns_to_ms(cpu_dt_ns),
                                               ns_to_ms(frame_dt_ns)));
            }
        }
    }
}
