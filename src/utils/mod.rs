//! Some emulator-related utils

pub mod screen;
pub mod clocks;
pub mod events;
pub mod smallnum;
pub use self::smallnum::*;
pub use self::events::*;
pub use self::clocks::*;

#[derive(Copy, Clone)]
pub enum EmulationSpeed {
    Definite(usize),
    Max,
}

/// converts nanoseconds to miliseconds
#[inline]
fn ns_to_ms(ns: u64) -> f64 {
    ns as f64 / 1_000_000f64
}
/// converts miliseconds to nanoseconds
#[inline]
fn ms_to_ns(s: f64) -> u64 {
    (s * 1_000_000_f64) as u64
}

/// Internal function for making word from 2 bytes
#[inline]
pub fn make_word(hi: u8, lo: u8) -> u16 {
    ((hi as u16) << 8) | (lo as u16)
}

/// Internal function for splitting word in two bytes
#[inline]
pub fn split_word(value: u16) -> (u8, u8) {
    ((value >> 8) as u8, value as u8)
}

/// preforms word displacement
#[inline]
pub fn word_displacement(word: u16, d: i8) -> u16 {
    (word as i32).wrapping_add(d as i32) as u16
}

// TODO: Check how normal Rust type conversion works
/// transforms bool to u8
#[inline]
pub fn bool_to_u8(value: bool) -> u8 {
    match value {
        true => 1,
        false => 0,
    }
}
