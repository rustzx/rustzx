//! RustZX - ZX Spectum emulator
//! Copyright (c) 2016 Vladislav Nikonov
//! The MIT License (MIT)
//! View full License text in file `LICENSE.md`
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
/// library for vector/matrix math
extern crate cgmath;

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
