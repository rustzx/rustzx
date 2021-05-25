#![no_std]
#![allow(dead_code)]

pub(crate) mod emulator;
pub(crate) mod settings;
pub(crate) mod utils;

pub mod error;
pub mod host;
pub mod zx;

pub use emulator::{Emulator, Stopwatch};
pub use settings::RustzxSettings;
pub use utils::EmulationSpeed;

extern crate alloc;

pub type Result<T> = core::result::Result<T, error::Error>;
