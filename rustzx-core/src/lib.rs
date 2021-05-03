#![no_std]
#![allow(dead_code)]

pub mod emulator;
pub mod error;
pub mod host;
pub mod settings;
pub mod utils;
pub mod z80;
pub mod zx;

// TODO: eliminate need for alloc (optional feature?)
extern crate alloc;

pub use crate::error::Error;
pub type Result<T> = core::result::Result<T, Error>;
