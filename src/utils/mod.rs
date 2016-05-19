//! Some emulator-related utils

pub mod smallnum;
pub use self::smallnum::*;

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
