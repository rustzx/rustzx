#![allow(dead_code)]

// glium as graphics library
#[macro_use]
extern crate glium;
// time lib for frame timings
extern crate time;
// Lazy static for macine specs
#[macro_use]
extern crate lazy_static;

// crate consists of theese modules
mod utils;
mod z80;
mod zx;
mod app;

use app::RustZXApp;

fn main() {
    let mut app = RustZXApp::new();
    app.start();
}
