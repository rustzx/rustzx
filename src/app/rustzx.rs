//! Main application class module
//! Handles all platform-related, hardware-related stuff
use std::fs::*;
use std::io::Write;

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
    pub emulator: Emulator,
    pub snd: SoundThread<'static>,
}

impl RustZXApp {
    /// Returns new application instance
    pub fn new() -> RustZXApp {
        let emu = Emulator::new(ZXMachine::Sinclair48K);
        RustZXApp {
            emulator: emu,
            snd: SoundThread::new(),
        }
    }
    /// starts application
    pub fn start(&mut self) {
        // build new glium window
        self.snd.run_sound_thread();
        let display = WindowBuilder::new()
                          .with_dimensions(SCREEN_WIDTH as u32 * 2, SCREEN_HEIGHT as u32 * 2)
                          .build_glium()
                          .unwrap();
        let renderer = ZXScreenRenderer::new(&display);
        'render_loop: loop {
            let frame_target_dt_ns = ms_to_ns((1000 / 50) as f64);
            let frame_start_ns = time::precise_time_ns();
            // emulation loop
            let cpu_dt_ns = self.emulator.emulate_frame(MAX_FRAME_TIME_NS);
            loop {
                if let Some(sample) = self.emulator.controller.beeper.pop() {
                    self.snd.send(sample);
                } else {
                    break;
                }
            }
            renderer.draw_screen(&display,
                self.emulator.controller.get_border_texture(),
                self.emulator.controller.get_canvas_texture());
            for event in display.poll_events() {
                match event {
                    Event::Closed => {
                        break 'render_loop;
                    }
                    Event::KeyboardInput(state, _, Some(key_code)) => {
                        match key_code {
                            VKey::Insert => {
                                self.emulator.controller.play_tape();
                            }
                            VKey::Delete => {
                                self.emulator.controller.stop_tape();
                            }
                            VKey::F2 => {
                                let mut f = File::create("/home/pacmancoder/snap.rustzx").unwrap();
                                f.write_all(&self.emulator.controller.dump()).unwrap();
                            }
                            VKey::F3 => {
                                self.emulator.set_speed(EmulationSpeed::Definite(1))
                            }
                            VKey::F4 => {
                                self.emulator.set_speed(EmulationSpeed::Definite(1))
                            }
                            VKey::F5 => {
                                self.emulator.set_speed(EmulationSpeed::Max)
                            }
                            _ => {
                                if let Some(key) = vkey_to_zxkey(key_code) {
                                    match state {
                                        KeyState::Pressed => self.emulator.controller.send_key(key, true),
                                        KeyState::Released => self.emulator.controller.send_key(key, false),
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            let emulation_dt_ns = time::precise_time_ns() - frame_start_ns;

            // wait some time for 50 FPS

            // if emulation_dt_ns < frame_target_dt_ns {
            //     thread::sleep(Duration::new(0, (frame_target_dt_ns - emulation_dt_ns) as u32));
            // };
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
