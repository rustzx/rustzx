//! This module provides main application class and modules `rustzx`, `video`, `keyboard`
// module - parts
mod rustzx;
mod video;
mod keyboard;
mod sound_thread;
// main re-export
pub use self::rustzx::RustZXApp;
