//! Collection of independent types, used by multiplie other types

#[derive(Clone, Copy, PartialEq, Eq)]
#[cfg_attr(rustfmt, rustfmt_skip)]
/// 1 bit unsigned
pub enum U1 {
    N0, N1,
}
impl U1 {
    pub fn from_byte(value: u8, shift: u8) -> U1 {
        assert!(shift < 8);
        match (value >> shift) & 1 {
            0 => U1::N0,
            _ => U1::N1,
        }
    }
    pub fn as_byte(self) -> u8 {
        match self {
            U1::N0 => 0,
            U1::N1 => 1,
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq)]
#[cfg_attr(rustfmt, rustfmt_skip)]
/// 2 bit unsigned
pub enum U2 {
    N0, N1, N2, N3,
}
impl U2 {
    pub fn from_byte(value: u8, shift: u8) -> U2 {
        assert!(shift < 7);
        match (value >> shift) & 3 {
            0 => U2::N0,
            1 => U2::N1,
            2 => U2::N2,
            _ => U2::N3,
        }
    }
    pub fn as_byte(self) -> u8 {
        match self {
            U2::N0 => 0,
            U2::N1 => 1,
            U2::N2 => 2,
            U2::N3 => 3,
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq)]
#[cfg_attr(rustfmt, rustfmt_skip)]
/// 3 bit unsigned
pub enum U3 {
    N0, N1, N2, N3, N4, N5, N6, N7,
}
impl U3 {
    pub fn from_byte(value: u8, shift: u8) -> U3 {
        assert!(shift < 6);
        match (value >> shift) & 7 {
            0 => U3::N0,
            1 => U3::N1,
            2 => U3::N2,
            3 => U3::N3,
            4 => U3::N4,
            5 => U3::N5,
            6 => U3::N6,
            _ => U3::N7,
        }
    }
    pub fn as_byte(self) -> u8 {
        match self {
            U3::N0 => 0,
            U3::N1 => 1,
            U3::N2 => 2,
            U3::N3 => 3,
            U3::N4 => 4,
            U3::N5 => 5,
            U3::N6 => 6,
            U3::N7 => 7,
        }
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
pub enum IntMode {
    IM0,
    IM1,
    IM2,
}
