use z80::*;
/// Operand for 8-bit LD instructions
pub enum LoadOperand8 {
    Indirect(u16),
    Reg(RegName8),
}

/// Operand for 8-bit Rotate instructions
pub enum RotOperand8  {
    Indirect(u16),
    Reg(RegName8),
}

/// Operand for 8-bit ALU instructions
pub enum AluOperand8 {
    Indirect(u16),
    Reg(RegName8),
    Const(u8),
}

/// Direction of address cahange in block functions
pub enum BlockDir {
    Inc,
    Dec,
}

/// Execution result of the instruction
pub enum Clocks {
    /// contains clocks count, which was elapsed to execute instruction
    Some(u8),
    /// emulator fail
    ExecFail,
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
    /// split opcode into parts
    pub fn from_byte(data: u8) -> Opcode {
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
