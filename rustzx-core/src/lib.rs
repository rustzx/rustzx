#![no_std]

pub(crate) mod emulator;
pub(crate) mod settings;
pub(crate) mod utils;

pub mod error;
pub mod host;
pub mod zx;

pub use emulator::{poke, EmulationInfo, EmulationStopReason, Emulator};
pub use settings::RustzxSettings;
pub use utils::EmulationMode;

#[cfg(feature = "strum")]
pub use strum::IntoEnumIterator as IterableEnum;

extern crate alloc;

pub type Result<T> = core::result::Result<T, error::Error>;
