// TABLES FORM FUSE ZX SPECTRUM EMULATOR:

// [FUSE] Whether a half carry occurred or not can be determined by looking at
// the 3rd bit of the two arguments and the result; these are hashed
// into this table in the form r12, where r is the 3rd bit of the
// result, 1 is the 3rd bit of the 1st argument and 2 is the
// third bit of the 2nd argument; the tables differ for add and subtract
// operations
pub const HALF_CARRY_ADD_TABLE: [u8; 8] = [ 0, 1, 1, 1, 0, 0, 0, 1 ];
pub const HALF_CARRY_SUB_TABLE: [u8; 8] = [ 0, 0, 1, 0, 1, 0, 1, 1 ];
// [FUSE] Similarly, overflow can be determined by looking at the 7th bits; again
// the hash into this table is r12
pub const OVERFLOW_ADD_TABLE: [u8; 8] = [ 0, 0, 0, 1, 1, 0, 0, 0 ];
pub const OVERFLOW_SUB_TABLE: [u8; 8] = [ 0, 1, 0, 0, 0, 0, 1, 0 ];

/// TODO: docs
pub fn lookup8_r12(a: u8, b: u8, r: u8) -> u8 {
    return ((a & 0x88) >> 3) | ((b & 0x88) >> 2) | ((r & 0x88) >> 1);
}

pub fn lookup16_r12(a: u16, b: u16, r: u16) -> u8 {
    return (((a & 0x8800) >> 11) | ((b & 0x8800) >> 10) | ((r & 0x8800) >> 9)) as u8;
}
