//! Collection of independent types, used by multiplie other types
use std::ops::AddAssign;

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
    pub fn count(&self) -> usize {
        self.0
    }
}


/// Instruction prefix type
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Prefix {
    None,
    CB,
    DD,
    ED,
    FD,
}
impl Prefix {
    /// get prefix option from byte value
    pub fn from_byte(data: u8) -> Prefix {
        match data {
            0xCB => Prefix::CB,
            0xDD => Prefix::DD,
            0xED => Prefix::ED,
            0xFD => Prefix::FD,
            _ => Prefix::None,
        }
    }
}

/// Interrupt mode
#[derive(Debug, Clone, Copy)]
pub enum IntMode {
    IM0,
    IM1,
    IM2,
}
