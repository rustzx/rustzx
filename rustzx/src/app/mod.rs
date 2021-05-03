//! This module provides main application class.
mod events;
mod rustzx;
mod sound;
mod video;
mod settings;

// main re-export
pub use self::rustzx::RustzxApp;
pub use self::settings::Settings;
