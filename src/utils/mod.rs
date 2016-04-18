pub mod smallnum;
pub use self::smallnum::*;

use std::{i8, i16};

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

// TODO: REMOVE HALF_CARRY AND OVERFLOW CHECK FUNCTIONS, REPLACE WITH LOOKUP TABLES

/// check half_carry after 16 bit addition
#[inline]
pub fn half_carry_16(a: u16, b: u16) -> bool {
    ((a & 0xFFF) + (b & 0xFFF)) > 0xFFF
}
/// check half_carry after 8 bit addition
#[inline]
pub fn half_carry_8(a: u8, b: u8) -> bool {
    ((a & 0xF) + (b & 0xF)) > 0xF
}
/// check half_carry after 8 bit subtraction
#[inline]
pub fn half_borrow_16(a: u16, b: u16) -> bool {
    (a & 0xFFF) < (b & 0xFFF)
}
/// check half_carry after 8 bit subtraction
#[inline]
pub fn half_borrow_8(a: u8, b: u8) -> bool {
    (a & 0xF) < (b & 0xF)
}

/// preforms word displacement
#[inline]
pub fn word_displacement(word: u16, d: i8) -> u16 {
    (word as i32).wrapping_add(d as i32) as u16
}

/// checks signed 8-bit overflow after addition
#[inline]
pub fn check_add_overflow_8(a: i8, b: i8) -> bool {
    let c = a as i16 + b as i16;
    if (c > i8::MAX as i16) || (c < i8::MIN as i16) {
        true
    } else {
        false
    }
}

/// checks signed 16-bit overflow after addition
#[inline]
pub fn check_add_overflow_16(a: i16, b: i16) -> bool {
    let c = a as i32 + b as i32;
    if (c > i16::MAX as i32) || (c < i16::MIN as i32) {
        true
    } else {
        false
    }
}

/// checks signed 8-bit overflow after subtraction
#[inline]
pub fn check_sub_overflow_8(a: i8, b: i8) -> bool {
    let c = a as i16 - b as i16;
    if (c > i8::MAX as i16) || (c < i8::MIN as i16) {
        true
    } else {
        false
    }
}

/// checks signed 16-bit overflow after subtraction
#[inline]
pub fn check_sub_overflow_16(a: i16, b: i16) -> bool {
    let c = a as i32 - b as i32;
    if (c > i16::MAX as i32) || (c < i16::MIN as i32) {
        true
    } else {
        false
    }
}

/// transforms bool to u8
#[inline]
pub fn bool_to_u8(value: bool) -> u8 {
    match value {
        true => 1,
        false => 0,
    }
}

/// check bit in value
#[inline]
pub fn bit(value: u8, bit: u8) -> bool {
    (value & (0x01 << bit)) != 0
}
