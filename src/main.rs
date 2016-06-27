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

use app::RustZXApp;

fn main() {
    RustZXApp::new().init().start();
}
