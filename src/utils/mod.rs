/// Internal function for making word from 2 bytes
pub fn make_word(hi: u8, lo: u8) -> u16 {
    ((hi as u16) << 8) | (lo as u16)
}

/// Internal function for splitting word in two bytes
pub fn split_word(value: u16) -> (u8, u8) {
    ((value >> 8) as u8, value as u8)
}
