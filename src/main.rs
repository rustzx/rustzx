#![allow(dead_code)]

#[macro_use]
extern crate glium;
extern crate time;

mod utils;
mod z80;
mod zx;
mod app;

use app::RustZXApp;

fn main() {
    let mut app = RustZXApp::new();
    app.start();
}
