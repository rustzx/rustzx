//! Contains internal emulator tables
use crate::z80::{FLAG_F3, FLAG_F5, FLAG_HALF_CARRY, FLAG_PV, FLAG_SIGN, FLAG_ZERO};
use lazy_static::lazy_static;

// TABLES FORM FUSE ZX SPECTRUM EMULATOR:

// [FUSE] Whether a half carry occurred or not can be determined by looking at
// the 3rd bit of the two arguments and the result; these are hashed
// into this table in the form r12, where r is the 3rd bit of the
// result, 1 is the 3rd bit of the 1st argument and 2 is the
// third bit of the 2nd argument; the tables differ for add and subtract
// operations
#[rustfmt::skip]
pub const HALF_CARRY_ADD_TABLE: [u8; 8] = [
    0, FLAG_HALF_CARRY, FLAG_HALF_CARRY, FLAG_HALF_CARRY,0, 0, 0, FLAG_HALF_CARRY
];
#[rustfmt::skip]
pub const HALF_CARRY_SUB_TABLE: [u8; 8] = [
    0, 0, FLAG_HALF_CARRY, 0, FLAG_HALF_CARRY, 0, FLAG_HALF_CARRY, FLAG_HALF_CARRY
];

// [FUSE] Similarly, overflow can be determined by looking at the 7th bits; again
// the hash into this table is r12
pub const OVERFLOW_ADD_TABLE: [u8; 8] = [0, 0, 0, FLAG_PV, FLAG_PV, 0, 0, 0];
pub const OVERFLOW_SUB_TABLE: [u8; 8] = [0, FLAG_PV, 0, 0, 0, 0, FLAG_PV, 0];

/// Returns pv/hc flags lookup id from 8-bit operands
pub fn lookup8_r12(a: u8, b: u8, r: u8) -> u8 {
    ((a & 0x88) >> 3) | ((b & 0x88) >> 2) | ((r & 0x88) >> 1)
}
/// Returns pv/hc flags lookup id from 16-bit operands
pub fn lookup16_r12(a: u16, b: u16, r: u16) -> u8 {
    (((a & 0x8800) >> 11) | ((b & 0x8800) >> 10) | ((r & 0x8800) >> 9)) as u8
}

/// Parity table, internal
#[rustfmt::skip]
const PARITY_BIT: [u8; 256] = [
	1, 0, 0, 1, 0, 1, 1, 0, 0, 1, 1, 0, 1, 0, 0, 1,
	0, 1, 1, 0, 1, 0, 0, 1, 1, 0, 0, 1, 0, 1, 1, 0,
	0, 1, 1, 0, 1, 0, 0, 1, 1, 0, 0, 1, 0, 1, 1, 0,
	1, 0, 0, 1, 0, 1, 1, 0, 0, 1, 1, 0, 1, 0, 0, 1,
	0, 1, 1, 0, 1, 0, 0, 1, 1, 0, 0, 1, 0, 1, 1, 0,
	1, 0, 0, 1, 0, 1, 1, 0, 0, 1, 1, 0, 1, 0, 0, 1,
	1, 0, 0, 1, 0, 1, 1, 0, 0, 1, 1, 0, 1, 0, 0, 1,
	0, 1, 1, 0, 1, 0, 0, 1, 1, 0, 0, 1, 0, 1, 1, 0,
	0, 1, 1, 0, 1, 0, 0, 1, 1, 0, 0, 1, 0, 1, 1, 0,
	1, 0, 0, 1, 0, 1, 1, 0, 0, 1, 1, 0, 1, 0, 0, 1,
	1, 0, 0, 1, 0, 1, 1, 0, 0, 1, 1, 0, 1, 0, 0, 1,
	0, 1, 1, 0, 1, 0, 0, 1, 1, 0, 0, 1, 0, 1, 1, 0,
	1, 0, 0, 1, 0, 1, 1, 0, 0, 1, 1, 0, 1, 0, 0, 1,
	0, 1, 1, 0, 1, 0, 0, 1, 1, 0, 0, 1, 0, 1, 1, 0,
	0, 1, 1, 0, 1, 0, 0, 1, 1, 0, 0, 1, 0, 1, 1, 0,
	1, 0, 0, 1, 0, 1, 1, 0, 0, 1, 1, 0, 1, 0, 0, 1,
];

// Generated table of parity flag setting
lazy_static! {
    pub static ref PARITY_TABLE: [u8; 256] = {
        let mut arr = [0u8; 256];
        for (n, x) in arr.iter_mut().enumerate() {
            *x = PARITY_BIT[n] * FLAG_PV;
        }
        arr
    };
}

// Generated table of F3 and F5 flags
lazy_static! {
    pub static ref F3F5_TABLE: [u8; 256] = {
        let mut arr = [0u8; 256];
        for (n, x) in arr.iter_mut().enumerate() {
            *x = (n as u8) & (FLAG_F3 | FLAG_F5);
        }
        arr
    };
}

// Generated table of F3,F5,Z,S flags
lazy_static! {
    pub static ref SZF3F5_TABLE: [u8; 256] = {
        let mut arr = [0u8; 256];
        for (n, x) in arr.iter_mut().enumerate() {
            *x = (n as u8) & (FLAG_F3 | FLAG_F5 | FLAG_SIGN);
            if n == 0 {
                *x |= FLAG_ZERO;
            };
        }
        arr
    };
}

// Generated table of F3,F5,Z,S flags
lazy_static! {
    pub static ref SZPF3F5_TABLE: [u8; 256] = {
        let mut arr = [0u8; 256];
        for (n, x) in arr.iter_mut().enumerate() {
            *x = (n as u8) & (FLAG_F3 | FLAG_F5 | FLAG_SIGN);
            *x |= PARITY_BIT[n] * FLAG_PV;
            if n == 0 {
                *x |= FLAG_ZERO;
            };
        }
        arr
    };
}
