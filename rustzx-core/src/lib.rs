#![no_std]

pub(crate) mod emulator;
pub(crate) mod settings;
pub(crate) mod utils;

pub mod error;
pub mod host;
pub mod zx;

pub use emulator::Emulator;
pub use settings::RustzxSettings;
pub use utils::EmulationMode;

// TODO(#118) Use no_std for strum in rustzx-core
#[cfg(feature = "strum")]
pub use strum::IntoEnumIterator as IterableEnum;

extern crate alloc;

pub type Result<T> = core::result::Result<T, error::Error>;
