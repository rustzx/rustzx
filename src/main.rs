#![allow(dead_code)]

#[macro_use]
extern crate glium;
extern crate time;
#[macro_use]
extern crate lazy_static;
mod utils;
mod z80;
mod zx;
mod app;

use app::RustZXApp;

fn main() {
    let mut app = RustZXApp::new();
    app.start();
}
