//! Some emulator-related utils

pub mod screen;

#[derive(Copy, Clone)]
pub enum EmulationMode {
    FrameCount(usize),
    Max,
}
