#![cfg_attr(all(not(test), not(feature = "std")), no_std)]
#![allow(dead_code)]

pub mod emulator;
pub mod settings;
pub mod utils;
pub mod z80;
pub mod zx;

extern crate alloc;
