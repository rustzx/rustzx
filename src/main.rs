#![allow(dead_code)]

/// glium as graphics library
#[macro_use]
extern crate glium;
/// time lib for frame timings
extern crate time;
/// Lazy static for macine specs
#[macro_use]
extern crate lazy_static;
/// Command line parser
extern crate clap;
/// Library for sound rendering
extern crate portaudio;

// crate consists of theese modules
mod utils;
mod z80;
mod zx;
mod app;
mod emulator;

use std::path::Path;

use clap::{Arg, App, AppSettings};
use app::RustZXApp;
use utils::EmulationSpeed;

fn main() {
    let mut app = RustZXApp::new();
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
                            .help("Disables sound. Use it when you have problems with audio
                                   playback"))
                        .get_matches();
    // check command line args
    // TODO: move main contol routines up to `Emulator`
    // use default rom
    app.emulator.controller.load_default_rom();
    // but load another if requested
    if let Some(path) = cmd.value_of("ROM") {
        if Path::new(path).is_file() {
            app.emulator.controller.load_rom(path);
        } else {
            println!("[Warning] ROM file \"{}\" not found", path);
        }
    }
    // TAP files
    if let Some(path) = cmd.value_of("TAP") {
        if Path::new(path).is_file() {
            app.emulator.controller.insert_tape(path);
        } else {
            println!("[Warning] Tape file \"{}\" not found", path);
        }
    }
    // Tape fast loading flag
    app.emulator.set_fast_load(cmd.is_present("FAST_LOAD"));
    // SNA files
    if let Some(path) = cmd.value_of("SNA") {
        if Path::new(path).is_file() {
            app.emulator.load_sna(path);
        } else {
            println!("[Warning] Snapshot file \"{}\" not found", path);
        }
    }
    // set speed
    if let Some(speed_str) = cmd.value_of("SPEED") {
        if let Ok(speed) = speed_str.parse::<usize>() {
            app.emulator.set_speed(EmulationSpeed::Definite(speed));
        }
    }
    app.start();
}
