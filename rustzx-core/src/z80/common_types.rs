//! Collection of independent types, used by other types of z80 module

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
