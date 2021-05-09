//! This module provides main application class.
mod events;
mod rustzx;
mod settings;
mod sound;
pub(crate) mod video;

// main re-export
pub use self::{rustzx::RustzxApp, settings::Settings};
