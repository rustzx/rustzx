//! Small pack of useful functions

/// Internal function for making word from 2 bytes
pub fn make_word(hi: u8, lo: u8) -> u16 {
    ((hi as u16) << 8) | (lo as u16)
}

/// Internal function for splitting word in two bytes
pub fn split_word(value: u16) -> (u8, u8) {
    ((value >> 8) as u8, value as u8)
}

/// check half_carry after 16 bit addition
pub fn half_carry_16(a: u16, b: u16) -> bool {
    ((a & 0xFFF) + (b & 0xFFF)) > 0xFFF
}
/// check half_carry after 8 bit addition
pub fn half_carry_8(a: u8, b: u8) -> bool {
    ((a & 0xF) + (b & 0xF)) > 0xF
}
/// check half_carry after 8 bit subtraction
pub fn half_borrow_8(a: u8, b: u8) -> bool {
    (a & 0xF) < (b & 0xF)
}

/// preforms word displacement
pub fn word_displacement(word: u16, d: i8) -> u16 {
    let result = if d >= 0 {
        word.wrapping_add(d as u16)
    } else {
        word.wrapping_sub(d.abs() as u16)
    };
    result
}
