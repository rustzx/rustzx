//! This module provides main application class and modules `rustzx`, `video`, `keyboard`

mod rustzx;
mod video;
mod keyboard;
pub mod sound_thread;
pub use self::rustzx::RustZXApp;
