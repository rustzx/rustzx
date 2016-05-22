//! Main application class module
//! TODO: Code refactoring

use std::thread;
use std::fs::*;
use std::io::Write;
use std::time::Duration;
use std::io::Read;

use time;
use glium::glutin::{WindowBuilder, Event, ElementState as KeyState};
use glium::DisplayBuild;
use glium::glutin::VirtualKeyCode as VKey;

use app::video::ZXScreenRenderer;
use app::keyboard::vkey_to_zxkey;
use z80::*;
use zx::*;

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
        let mut trace = false;
        let mut controller = ZXController::new(ZXMachine::Sinclair48K);
        let mut cpu = Z80::new();
        let mut memory = ZXMemory::new(RomType::K16, RamType::K48);
        let mut tape = tape::Tap::new();
        tape.insert("/home/pacmancoder/test.tap");
        tape.play();
        let mut play = false;
        let mut rom = Vec::new();
        if let Ok(mut file) = File::open("/home/pacmancoder/48.rom") {
            file.read_to_end(&mut rom).unwrap();
        } else {
            panic!("ROM not found!");
        }
        memory.load_rom(0, &rom).unwrap();
        controller.atach_memory(memory);
        controller.attach_screen(ZXScreen::new(ZXMachine::Sinclair48K, ZXPalette::default()));
        // build new glium window
        let display = WindowBuilder::new()
                          .with_dimensions(SCREEN_WIDTH as u32 * 2, SCREEN_HEIGHT as u32 * 2)
                          .build_glium()
                          .unwrap();
        let mut renderer = ZXScreenRenderer::new(&display);
        // NOTE: 16x speed
        let mut frame_counter = 0_usize;
        let mut speed = 16u64;
        let mut frame_devider = 1;
        'render_loop: loop {
            let frame_target_dt_ns = ms_to_ns((1000 / (50 * speed)) as f64);
            frame_counter += 1;
            controller.new_frame();

            let frame_start_ns = time::precise_time_ns();
            // emulation loop
            if trace {
                println!("Frame start");
            }
            loop {
                let prev_clocks = controller.clocks();
                cpu.emulate(&mut controller);
                if cpu.regs.get_pc() == 0x556 {
                    println!("Tape Access!");
                }
                let clocks_delta = controller.clocks() - prev_clocks;
                if play {
                    tape.process_clocks(clocks_delta);
                }
                controller.set_ear(tape.current_bit());
                if controller.frame_finished() {
                    break;
                }
            }
            trace = false;
            let cpu_dt_ns = time::precise_time_ns() - frame_start_ns;
            if frame_counter % frame_devider == 0 {
                renderer.set_border_color(controller.get_border_color());
                renderer.draw_screen(&display, controller.get_screen_texture());
            }
            // glutin events
            for event in display.poll_events() {
                match event {
                    Event::Closed => {
                        break 'render_loop;
                    }
                    Event::KeyboardInput(state, _, Some(key_code)) => {
                        match key_code {
                            VKey::Insert => {
                                play = true;
                            }
                            VKey::Delete => {
                                play = false;
                            }
                            VKey::F2 => {
                                let mut f = File::create("/home/pacmancoder/snap.rustzx").unwrap();
                                f.write_all(&controller.dump()).unwrap();
                            }
                            VKey::F3 => {
                                trace = true;
                            }
                            VKey::F4 => {
                                speed = 1;
                                frame_devider = 1;
                            }
                            VKey::F5 => {
                                speed = 128;
                                frame_devider = 16;
                            }
                            _ => {
                                if let Some(key) = vkey_to_zxkey(key_code) {
                                    match state {
                                        KeyState::Pressed => controller.send_key(key, true),
                                        KeyState::Released => controller.send_key(key, false),
                                    }
                                }
                            }
                        }
                    }
                    Event::MouseWheel(_) => {
                        let pc = cpu.regs.get_pc();
                        println!("pc: {:#04X}", pc);
                    }
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
