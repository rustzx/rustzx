// The MIT License (MIT)
//
// Copyright (c) 2016 Vladislav Nikonov
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

//! RustZX - ZX Spectum emulator
//! Copyright (c) 2016 Vladislav Nikonov
//! The MIT License (MIT)
//! View full License text in file `LICENSE.md`
#![allow(dead_code)]

/// Lazy static for macine specs
#[macro_use]
extern crate lazy_static;
/// AY chip emulation library pacmancoder/rust-ayumi
extern crate ayumi;
/// Command line parser
extern crate clap;
/// backend => sound, video, events
extern crate sdl2;

// crate consists of theese modules
mod app;
mod backends;
mod emulator;
mod settings;
mod utils;
mod z80;
mod zx;

use app::RustzxApp;
use settings::RustzxSettings;
fn main() {
    let settings = RustzxSettings::from_clap();
    RustzxApp::from_config(settings).start();
}
