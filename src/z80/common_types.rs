//! Collection of independent types, used by other types of z80 module
use std::ops::AddAssign;

/// Clocks count
#[derive(Clone, Copy)]
pub struct Clocks(pub usize);

impl AddAssign for Clocks {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl AddAssign<usize> for Clocks {
    fn add_assign(&mut self, rhs: usize) {
        self.0 += rhs;
    }
}

impl Clocks {
    /// returns inner `usize` value
    pub fn count(&self) -> usize {
        self.0
    }
}


/// Instruction prefix type
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

/// Interrupt mode enum
#[derive(Debug, Clone, Copy)]
pub enum IntMode {
    IM0,
    IM1,
    IM2,
}
