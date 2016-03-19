//! Collection of independent types, used by multiplie other types

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
