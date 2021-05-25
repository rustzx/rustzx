/// preforms word displacement
#[inline]
pub fn word_displacement(word: u16, d: i8) -> u16 {
    (word as i32).wrapping_add(d as i32) as u16
}
