#![cfg_attr(all(not(test), not(feature = "std")), no_std)]
#![allow(dead_code)]

pub mod emulator;
pub mod settings;
pub mod utils;
pub mod z80;
pub mod zx;
pub mod host;
pub mod error;

extern crate alloc;

pub use crate::error::Error;
pub type Result<T> = core::result::Result<T, Error>;
