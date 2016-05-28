//! Main application class module
//! TODO: Code refactoring

use std::thread;
use std::fs::*;
use std::io::Write;
use std::time::Duration;

use time;
use glium::glutin::{WindowBuilder, Event, ElementState as KeyState};
use glium::DisplayBuild;
use glium::glutin::VirtualKeyCode as VKey;

use app::video::ZXScreenRenderer;
use app::keyboard::vkey_to_zxkey;
use zx::*;
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
pub struct RustZXApp;

impl RustZXApp {
    /// Returns new application instance
    pub fn new() -> RustZXApp {
        RustZXApp
    }
    /// starts application
    pub fn start(&mut self) {
        // build new glium window
        let mut emulator = Emulator::new(ZXMachine::Sinclair48K);
        emulator.controller.load_rom("/home/pacmancoder/48.rom");
        emulator.controller.insert_tape("/home/pacmancoder/test.tap");
        let display = WindowBuilder::new()
                          .with_dimensions(SCREEN_WIDTH as u32 * 2, SCREEN_HEIGHT as u32 * 2)
                          .build_glium()
                          .unwrap();
        let mut renderer = ZXScreenRenderer::new(&display);
        // NOTE: 16x speed
        //let mut frame_counter = 0_usize;
        let mut speed = EmulationSpeed::Definite(1);
        //let mut frame_devider = 1;
        'render_loop: loop {
            emulator.set_speed(speed);
            let frame_target_dt_ns = ms_to_ns((1000 / 50) as f64);
            //frame_counter += 1;

            let frame_start_ns = time::precise_time_ns();
            // emulation loop
            let cpu_dt_ns = emulator.emulate_frame(MAX_FRAME_TIME_NS);

            //if frame_counter % frame_devider == 0 {
            renderer.set_border_color(emulator.controller.get_border_color());
            renderer.draw_screen(&display, emulator.controller.get_screen_texture());
            //}
            // glutin events
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
                                let mut f = File::create("/home/pacmancoder/snap.rustzx").unwrap();
                                f.write_all(&emulator.controller.dump()).unwrap();
                            }
                            VKey::F3 => {
                                speed = EmulationSpeed::Definite(1);
                            }
                            VKey::F4 => {
                                speed = EmulationSpeed::Definite(16);
                            }
                            VKey::F5 => {
                                speed = EmulationSpeed::Max;
                            }
                            _ => {
                                if let Some(key) = vkey_to_zxkey(key_code) {
                                    match state {
                                        KeyState::Pressed => emulator.controller.send_key(key, true),
                                        KeyState::Released => emulator.controller.send_key(key, false),
                                    }
                                }
                            }
                        }
                    }
                    // Event::MouseWheel(_) => {
                    //     let pc = cpu.regs.get_pc();
                    //     println!("pc: {:#04X}", pc);
                    // }
                    _ => {}
                }
            }
            let emulation_dt_ns = time::precise_time_ns() - frame_start_ns;

            // wait some time for 50 FPS

            if emulation_dt_ns < frame_target_dt_ns {
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
