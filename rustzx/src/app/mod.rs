//! This module provides main application class.
mod events;
//mod rustzx;
mod settings;
pub mod sound;
pub(crate) mod video;

// main re-export
pub use self::settings::{Settings, SoundBackend};
