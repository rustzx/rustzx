#![allow(dead_code)]

// glium as graphics library
#[macro_use]
extern crate glium;
// time lib for frame timings
extern crate time;
// Lazy static for macine specs
#[macro_use]
extern crate lazy_static;
// Command line parser
extern crate clap;

// crate consists of theese modules
mod utils;
mod z80;
mod zx;
mod app;
mod emulator;

use clap::{Arg, App};
use app::RustZXApp;

fn main() {
    let mut app = RustZXApp::new();
    let cmd = App::new("rustzx")
                        .version(env!("CARGO_PKG_VERSION"))
                        .author("Vladislav Nikonov <pacmancoder@gmail.com>")
                        .about("ZX Spectrum emulator written in pure Rust")
                        .arg(Arg::with_name("ROM")
                            .short("r")
                            .long("rom")
                            .value_name("ROM_PATH")
                            .help("Selects path to rom, otherwise default will be used"))
                        .arg(Arg::with_name("TAP")
                            .short("t")
                            .long("tap")
                            .value_name("TAP_PATH")
                            .help("Selects path to *.tap file"))
                        .arg(Arg::with_name("FAST_LOAD")
                            .short("f")
                            .long("fastload")
                            .help("Accelerates standard tape loaders"))
                        .get_matches();
    // check command line args
    if let Some(path) = cmd.value_of("ROM") {
        // TODO: check path, if not exists -> load default ROM
        app.emulator.controller.load_rom(path);
    } else {
        app.emulator.controller.load_default_rom();
    }
    if let Some(path) = cmd.value_of("TAP") {
        // TODO: check path, if not exists -> print error, and load nothing
        app.emulator.controller.insert_tape(path);
    }
    app.emulator.set_fast_load(cmd.is_present("FAST_LOAD"));
    //app.emulator.controller.insert_tape("/home/pacmancoder/test.tap");
    app.start();
}
