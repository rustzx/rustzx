//! Some emulator-related utils

pub mod screen;

#[derive(Copy, Clone)]
pub enum EmulationSpeed {
    Definite(usize),
    Max,
}
