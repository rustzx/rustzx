//! Collection of independent types, used by multiplie other types

#[derive(Clone, Copy, PartialEq, Eq)]
#[cfg_attr(rustfmt, rustfmt_skip)]
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

/// Conditions
#[derive(Clone, Copy)]
pub enum Condition {
    NonZero,
    Zero,
    NonCary,
    Cary,
    ParityOdd,
    ParityEven,
    SignPositive,
    SignNegative,
}
impl Condition {
    /// Returns condition encoded in 3 bits
    /// # Failures
    /// Returns `None` if value is bigger than `0b111` or equals `0b110`
    pub fn from_u3(code: U3) -> Condition {
        match code {
            U3::N0 => Condition::NonZero,
            U3::N1 => Condition::Zero,
            U3::N2 => Condition::NonCary,
            U3::N3 => Condition::Cary,
            U3::N4 => Condition::ParityOdd,
            U3::N5 => Condition::ParityEven,
            U3::N6 => Condition::SignPositive,
            U3::N7 => Condition::SignNegative,
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

#[derive(Clone, Copy)]
// Flags for F register
pub enum Flag {
    Carry,
    Sub,
    ParityOveflow,
    F3,
    HalfCarry,
    F5,
    Zero,
    Sign,
}
impl Flag {
    /// get flag mask
    pub fn mask(self) -> u8 {
        match self {
            Flag::Carry => 0b00000001,
            Flag::Sub => 0b00000010,
            Flag::ParityOveflow => 0b00000100,
            Flag::F3 => 0b00001000,
            Flag::HalfCarry => 0b00010000,
            Flag::F5 => 0b00100000,
            Flag::Zero => 0b01000000,
            Flag::Sign => 0b10000000,
        }
    }
}
