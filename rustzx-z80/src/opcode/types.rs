use crate::{
    smallnum::{U1, U2, U3},
    RegName8,
};

/// Instruction prefix type
#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Prefix {
    None,
    CB,
    DD,
    ED,
    FD,
}

impl Prefix {
    /// Returns prefix type from byte value
    pub fn from_byte(data: u8) -> Prefix {
        match data {
            0xCB => Prefix::CB,
            0xDD => Prefix::DD,
            0xED => Prefix::ED,
            0xFD => Prefix::FD,
            _ => Prefix::None,
        }
    }

    /// Transforms prefix back to byte
    pub fn to_byte(self) -> Option<u8> {
        match self {
            Prefix::DD => Some(0xDD),
            Prefix::FD => Some(0xFD),
            Prefix::ED => Some(0xED),
            Prefix::CB => Some(0xCB),
            Prefix::None => None,
        }
    }
}

/// Operand for 8-bit LD instructions
pub enum LoadOperand8 {
    Indirect(u16),
    Reg(RegName8),
}

/// Operand for 8-bit Bit instructions
pub enum BitOperand8 {
    Indirect(u16),
    Reg(RegName8),
}

/// Direction of address cahange in block functions
pub enum BlockDir {
    Inc,
    Dec,
}

/// Opcode, devided in parts
/// ```text
/// xxyyyzzz
/// xxppqzzz
/// ```
/// Used for splitting opcode byte into parts
#[derive(Clone, Copy)]
pub struct Opcode {
    pub byte: u8,
    pub x: U2,
    pub y: U3,
    pub z: U3,
    pub p: U2,
    pub q: U1,
}
impl Opcode {
    /// splits opcode into parts
    pub(crate) fn from_byte(data: u8) -> Opcode {
        Opcode {
            byte: data,
            x: U2::from_byte(data, 6),
            y: U3::from_byte(data, 3),
            z: U3::from_byte(data, 0),
            p: U2::from_byte(data, 4),
            q: U1::from_byte(data, 3),
        }
    }
}
