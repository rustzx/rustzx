//! Main application class module
//! Handles all platform-related, hardware-related stuff
//! and command-line interface
//! TODO: make rustzxbuilder

use std::thread;
use std::time::Duration;
use std::path::Path;
use time;
use clap::{Arg, App, AppSettings};
// use glium::glutin::{WindowBuilder, Event, ElementState as KeyState, VirtualKeyCode as VKey};
// use glium::DisplayBuild;
//use app::video::ZXScreenRenderer;
//use app::keyboard::vkey_to_zxkey;
use app::sound::*;
use app::video::*;
use app::events::*;
use zx::*;
use zx::constants::*;
use zx::sound::ay::ZXAYMode;
use settings::RustzxSettings;
use emulator::*;
use utils::EmulationSpeed;


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
    emulator: Option<Emulator>,
    /// Sound rendering in a separate thread
    snd: Option<Box<SoundDevice>>,
    video: Option<Box<VideoDevice>>,
    events: Option<Box<EventDevice>>,
    tex_border: Option<TextureInfo>,
    tex_canvas: Option<TextureInfo>,
    settings: Option<RustzxSettings>,
    scale: u32,
}

impl RustzxApp {
    /// Returns new application instance
    pub fn new() -> RustzxApp {
        RustzxApp {
            emulator: None,
            snd: None,
            video: None,
            events: None,
            tex_border: None,
            tex_canvas: None,
            settings: None,
            scale: 2,
        }
    }

    /// Inits emulator, parses command line arguments
    pub fn init(&mut self) -> &mut Self {
        // Construction of App menu
        let cmd = App::new("rustzx")
                      .setting(AppSettings::ColoredHelp)
                      .version(env!("CARGO_PKG_VERSION"))
                      .author("Vladislav Nikonov <pacmancoder@gmail.com>")
                      .about("ZX Spectrum emulator written in pure Rust")
                      .arg(Arg::with_name("ROM")
                               .long("rom")
                               .value_name("ROM_PATH")
                               .help("Selects path to rom, otherwise default will be used"))
                      .arg(Arg::with_name("TAP")
                               .long("tap")
                               .value_name("TAP_PATH")
                               .help("Selects path to *.tap file"))
                      .arg(Arg::with_name("FAST_LOAD")
                               .short("f")
                               .long("fastload")
                               .help("Accelerates standard tape loaders"))
                      .arg(Arg::with_name("SNA")
                               .long("sna")
                               .value_name("SNA_PATH")
                               .help("Selects path to *.sna snapshot file"))
                      .arg(Arg::with_name("SPEED")
                               .long("speed")
                               .value_name("SPEED_VALUE")
                               .help("Selects speed for emulator in integer multiplier form"))
                      .arg(Arg::with_name("NO_SOUND")
                               .long("nosound")
                               .help("Disables sound. Use it when you have problems with audio \
                                      playback"))
                      .arg(Arg::with_name("128K")
                               .long("128k")
                               .help("Enables ZX Spectrum 128K mode"))
                      .arg(Arg::with_name("SCALE")
                                .long("scale")
                                .value_name("SCALE_VALUE")
                                .help("Selects default screen size. possible values are positive \
                                       integers. Default value is 2"))
                      .arg(Arg::with_name("AY")
                                .long("ay")
                                .value_name("AY_TYPE")
                                .possible_values(&["none", "mono", "abc", "acb"])
                                .help("Selects AY mode. Use none to disable. \
                                       For stereo features use abc or acb, default is mono."))
                      .arg(Arg::with_name("NOBEEPER")
                                .long("nobeeper")
                                .help("Disables beeper"))
                      .arg(Arg::with_name("VOLUME")
                                .long("volume")
                                .value_name("VOLUME_VALUE")
                                .help("Selects volume - value in range 0..200. Volume over 100 \
                                       can cause sound artifacts"))
                      .arg(Arg::with_name("LATENCY")
                                .long("latency")
                                .short("l")
                                .value_name("SAMPLES")
                                .help("Selects audio latency. Default is 1024 samples. Set higher \
                                       latency if emulator have sound glitches. Or if your \
                                       machine can handle this - try to set it lower. Must be \
                                       power of two."))
                      .arg(Arg::with_name("KEMPSTON")
                                .short("k")
                                .long("kempston")
                                .help("Enables Kempston joystick. Controlls via arrow keys and \
                                       Alt buttons"))
                      .get_matches();
        let mut settings = RustzxSettings::new();
        // select machine type
        if cmd.is_present("128K") {
            settings.machine(ZXMachine::Sinclair128K)
        } else {
            settings.machine(ZXMachine::Sinclair48K)
        };
        if cmd.is_present("KEMPSTON") {
            settings.use_kempston();
        }
        if let Some(value) = cmd.value_of("AY") {
            match value {
                "none" => { settings.ay(false) },
                "mono" => { settings.ay_mode(ZXAYMode::Mono) },
                "abc" => { settings.ay_mode(ZXAYMode::ABC) },
                "acb" => { settings.ay_mode(ZXAYMode::ACB) },
                _ => unreachable!(),
            };
        };
        if cmd.is_present("NOBEEPER") {
            settings.beeper(false);
        }
        if let Some(value) = cmd.value_of("VOLUME") {
            if let Ok(value) = value.parse::<usize>() {
                settings.volume(value);
            } else {
                println!("[Warning] Volume value is incorrect, setting volume to 100");
            }
        };
        let mut emulator = Emulator::new(&settings);
        // load another if requested
        if let Some(path) = cmd.value_of("ROM") {
            if Path::new(path).is_file() {
                emulator.controller.load_rom(path);
            } else {
                println!("[Warning] ROM file \"{}\" not found", path);
            }
        } else {
            // use default rom if custiom  ROM load failed
            emulator.controller.load_default_rom();
        }
        // TAP files
        if let Some(path) = cmd.value_of("TAP") {
            if Path::new(path).is_file() {
                emulator.controller.tape.insert(path);
            } else {
                println!("[Warning] Tape file \"{}\" not found", path);
            }
        }
        // Tape fast loading flag
        emulator.set_fast_load(cmd.is_present("FAST_LOAD"));
        // SNA files
        if let Some(path) = cmd.value_of("SNA") {
            if Path::new(path).is_file() {
                emulator.load_sna(path);
            } else {
                println!("[Warning] Snapshot file \"{}\" not found", path);
            }
        }
        // set speed
        if let Some(speed_str) = cmd.value_of("SPEED") {
            if let Ok(speed) = speed_str.parse::<usize>() {
                emulator.set_speed(EmulationSpeed::Definite(speed));
            }
        }
        // sound latency
        if let Some(latency_str) = cmd.value_of("LATENCY") {
            if let Ok(latency) = latency_str.parse::<usize>() {
                settings.latency(latency);
            }
        }
        // disable sound
        if cmd.is_present("NO_SOUND") {
            emulator.set_sound(false);
        } else {
            emulator.set_sound(true);
            self.snd = Some(Box::new(SoundSdl::new(&settings)));
        }
        // find out scale factor
        if let Some(scale_str) = cmd.value_of("SCALE") {
            if let Ok(mut scale) = scale_str.parse::<usize>() {
                // place into bounds
                if scale > 5 {
                    scale = 2;
                };
                settings.screen(SCREEN_WIDTH * scale, SCREEN_HEIGHT * scale);
                self.scale = scale as u32;
            } else {
                println!("[Warning] Invalid scale factor");
            };
        }
        // assemble
        let mut video = Box::new(VideoSdl::new(&settings));
        self.tex_border =  Some(video.gen_texture(SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32));
        self.tex_canvas =  Some(video.gen_texture(CANVAS_WIDTH as u32, CANVAS_HEIGHT as u32));
        self.video = Some(video);
        self.emulator = Some(emulator);
        self.events = Some(Box::new(EventsSdl::new(&settings)));
        self
    }

    /// Starts application itself
    pub fn start(&mut self) {
        let mut debug = false;
        if let Some(ref mut emulator) = self.emulator {
            if let Some(ref mut renderer) = self.video {
                'emulator: loop {
                    let frame_target_dt_ns = frame_length_ns(FPS);
                    // absolute start time
                    let frame_start_ns = time::precise_time_ns();
                    // Emulate all requested frames
                    let cpu_dt_ns = emulator.emulate_frames(MAX_FRAME_TIME_NS);
                    // if sound enabled sound ganeration allowed then move samples to sound thread
                    if let Some(ref mut snd) = self.snd {
                        // if can be turned off even on speed change, so check it everytime
                        if emulator.have_sound() {
                            loop {
                                if let Some(sample) = emulator.controller.mixer.pop() {
                                    snd.send_sample(sample);
                                } else {
                                    break;
                                }
                            }
                        }
                    }
                    // load new textures to sdl
                    renderer.update_texture(self.tex_border.unwrap(),
                                            emulator.controller.border.texture());
                    renderer.update_texture(self.tex_canvas.unwrap(),
                                            emulator.controller.canvas.texture());
                    // rendering block
                    renderer.begin();
                    renderer.draw_texture_2d(self.tex_border.unwrap(),
                                             Some(Rect::new(0,
                                                            0,
                                                            SCREEN_WIDTH as u32 * self.scale,
                                                            SCREEN_WIDTH as u32 * self.scale)));
                    renderer.draw_texture_2d(self.tex_canvas.unwrap(),
                                             Some(Rect::new(CANVAS_X as i32 * self.scale as i32,
                                                            CANVAS_Y as i32 * self.scale as i32,
                                                            CANVAS_WIDTH as u32 * self.scale,
                                                            CANVAS_HEIGHT as u32 * self.scale)));
                    renderer.end();
                    // check all events
                    if let Some(ref mut events) = self.events {
                        if let Some(event) = events.pop_event() {
                            match event {
                                Event::Exit => {
                                    break 'emulator;
                                }
                                Event::GameKey(key, state) => {
                                    emulator.controller.send_key(key, state);
                                }
                                Event::SwitchDebug => {
                                    debug = !debug;
                                    if !debug {
                                        renderer.set_title(&format!("RustZX v{}",
                                                                    env!("CARGO_PKG_VERSION")));
                                    }
                                }
                                Event::ChangeSpeed(speed) => {
                                    emulator.set_speed(speed);
                                }
                                Event::Kempston(key, state) => {
                                    if let Some(ref mut joy) = emulator.controller.kempston {
                                        joy.key(key, state);
                                    }
                                }
                                Event::InsertTape => {
                                    emulator.controller.tape.play()
                                }
                                Event::StopTape => {
                                    emulator.controller.tape.stop()
                                }
                            }
                        }
                    }
                    // how long emulation iteration was
                    let emulation_dt_ns = time::precise_time_ns() - frame_start_ns;
                    if emulation_dt_ns < frame_target_dt_ns  && !emulator.have_sound() {
                        // sleep untill frame sync
                        thread::sleep(Duration::new(
                            0, ((frame_target_dt_ns - emulation_dt_ns) as f64) as u32));
                    };
                    // get exceed clocks and use them on next iteration
                    let frame_dt_ns = time::precise_time_ns() - frame_start_ns;
                    // change window header
                    if debug {
                        renderer.set_title(&format!("CPU: {:7.3}ms; EMULATOR: {:7.3}ms; FRAME:{:7.3}ms",
                        ns_to_ms(cpu_dt_ns),
                        ns_to_ms(emulation_dt_ns),
                        ns_to_ms(frame_dt_ns)));
                    }
                }
            }
        }
    }
}
