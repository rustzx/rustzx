//! Main application class module
//! Handles all platform-related, hardware-related stuff
use std::thread;
use std::time::Duration;
use std::path::Path;
use std::io::Write;
use std::fs::File;

use time;
use glium::glutin::{WindowBuilder, Event, ElementState as KeyState};
use glium::DisplayBuild;
use glium::glutin::VirtualKeyCode as VKey;

use app::sound_thread::*;
use app::video::ZXScreenRenderer;
use app::keyboard::vkey_to_zxkey;
use zx::*;
use zx::constants::*;
use emulator::*;
use utils::EmulationSpeed;

use clap::{Arg, App, AppSettings};

// 50 ms
const MAX_FRAME_TIME_NS: u64 = 50 * 1000000;

/// converts nanoseconds  to miliseconds
fn ns_to_ms(ns: u64) -> f64 {
    ns as f64 / 1_000_000f64
}

/// converts miliseconds to nanoseconds
fn ms_to_ns(s: f64) -> u64 {
    (s * 1_000_000_f64) as u64
}

/// Application instance type
pub struct RustZXApp {
    /// main emulator object
    pub emulator: Option<Emulator>,
    /// Sound rendering in a separate thread
    snd: Option<SoundThread<'static>>,
}

impl RustZXApp {
    /// Returns new application instance
    pub fn new() -> RustZXApp {
        RustZXApp {
            emulator: None,
            snd: None,
        }
    }

    /// inits emulator, parses command line arguments
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
                               .help("Disables sound. Use it when you have problems with audio\
                                      playback"))
                      .arg(Arg::with_name("128K")
                               .long("128k")
                               .help("Enables ZX Spectrum 128K mode"))
                      .get_matches();
        let machine = if cmd.is_present("128K") {
            ZXMachine::Sinclair128K
        } else {
            ZXMachine::Sinclair48K
        };
        let mut emulator = Emulator::new(machine);
        // load another if requested
        if let Some(path) = cmd.value_of("ROM") {
            if Path::new(path).is_file() {
                emulator.controller.load_rom(path);
            } else {
                println!("[Warning] ROM file \"{}\" not found", path);
            }
        } else {
            // use default rom
            emulator.controller.load_default_rom();
        }
        // TAP files
        if let Some(path) = cmd.value_of("TAP") {
            if Path::new(path).is_file() {
                emulator.controller.insert_tape(path);
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
        // disable sound
        if cmd.is_present("NO_SOUND") {
            emulator.set_sound(false);
        } else {
            emulator.set_sound(true);
            self.snd = Some(SoundThread::new());
        }
        self.emulator = Some(emulator);
        self
    }

    /// starts application
    pub fn start(&mut self) {
        //let mut emulator = self.emulator.take().expect("[Error] start method invoked before init");
        // use sound if enabled
        if let Some(ref mut snd) = self.snd {
            snd.run_sound_thread();
        }
        if let Some(ref mut emulator) = self.emulator {
            // build new glium window
            let display = WindowBuilder::new()
                              .with_dimensions(SCREEN_WIDTH as u32 * 2, SCREEN_HEIGHT as u32 * 2)
                              .build_glium()
                              .ok()
                              .expect("[ERROR] Glium (OpenGL) initialization error");
            let renderer = ZXScreenRenderer::new(&display);
            'render_loop: loop {
                let frame_target_dt_ns = ms_to_ns((1000 / 50) as f64);
                let frame_start_ns = time::precise_time_ns();
                // emulation loop
                let cpu_dt_ns = emulator.emulate_frame(MAX_FRAME_TIME_NS);
                // if sound enabled sound ganeration allowed then move samples to sound thread
                if let Some(ref mut snd) = self.snd {
                    if emulator.have_sound() {
                        loop {
                            if let Some(sample) = emulator.controller.beeper.pop() {
                                snd.send(sample);
                            } else {
                                break;
                            }
                        }
                    }
                }
                renderer.draw_screen(&display,
                                     emulator.controller.get_border_texture(),
                                     emulator.controller.get_canvas_texture());
                for event in display.poll_events() {
                    match event {
                        Event::Closed => {
                            break 'render_loop;
                        }
                        Event::KeyboardInput(state, _, Some(key_code)) => {
                            match key_code {
                                VKey::Insert => {
                                    emulator.controller.play_tape();
                                }
                                VKey::Delete => {
                                    emulator.controller.stop_tape();
                                }
                                VKey::F2 => {
                                    let dump = emulator.controller.memory.dump();
                                    let mut file =
                                        File::create("/home/pacmancoder/rustzx_dump.bin").unwrap();
                                    file.write(&dump).unwrap();
                                }
                                VKey::F3 => emulator.set_speed(EmulationSpeed::Definite(1)),
                                VKey::F4 => emulator.set_speed(EmulationSpeed::Definite(2)),
                                VKey::F5 => emulator.set_speed(EmulationSpeed::Max),
                                _ => {
                                    if let Some(key) = vkey_to_zxkey(key_code) {
                                        match state {
                                            KeyState::Pressed => {
                                                emulator.controller.send_key(key, true)
                                            }
                                            KeyState::Released => {
                                                emulator.controller.send_key(key, false)
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
                let emulation_dt_ns = time::precise_time_ns() - frame_start_ns;

                // wait some time for 50 FPS if emulator syncs self not using sound callbacks
                if (emulation_dt_ns < frame_target_dt_ns) && !emulator.have_sound() {
                    thread::sleep(Duration::new(0, (frame_target_dt_ns - emulation_dt_ns) as u32));
                };
                let frame_dt_ns = time::precise_time_ns() - frame_start_ns;
                if let Some(wnd) = display.get_window() {
                    wnd.set_title(&format!("CPU: {:7.3}ms; EMULATOR: {:7.3}ms; FRAME:{:7.3}ms",
                                           ns_to_ms(cpu_dt_ns),
                                           ns_to_ms(emulation_dt_ns),
                                           ns_to_ms(frame_dt_ns)));
                }
            }
        }
    }
}
