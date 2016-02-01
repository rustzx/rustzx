
/// 8-bit registers names
#[cfg_attr(rustfmt, rustfmt_skip)]
enum RegName8 {
    A, F,
    B, C,
    D, E,
    H, L,
    IXH, IXL,
    IYH, IYL,
    I,
    R,
}
/// 16-bit registers names
enum RegName16 {
    PC,
    SP,
    AF,
    BC,
    DE,
    HL,
    IX,
    IY,
}

/// Conditions
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
/// Decoding cpu registers\conditions names.
struct NameDecoder;
impl NameDecoder {
    /// Gives 8 bit general purpose register name from code. Code must be lower than `0b1000`.
    /// Returns `Option::None` if code equals `0b110` (Flags)
    pub fn reg_gp_8(code: u8) -> Option<RegName8> {
        assert!(code <= 0b111, " Invalid (8 bit, gp) register index: {:#b}");
        match code {
            0b111 => Some(RegName8::A),
            0b000 => Some(RegName8::B),
            0b001 => Some(RegName8::C),
            0b010 => Some(RegName8::D),
            0b011 => Some(RegName8::E),
            0b100 => Some(RegName8::H),
            0b101 => Some(RegName8::L),
            _ => None,
        }
    }

    /// Gives 16 bit general purpose register name from code. Code must be lower than `0b100`
    pub fn reg_gp_16(code: u8) -> RegName16 {
        assert!(code <= 0b11, " Invalid (16 bit, gp) register index: {:#b}");
        match code {
            0b11 => RegName16::AF,
            0b00 => RegName16::BC,
            0b01 => RegName16::DE,
            0b10 => RegName16::HL,
            _ => unreachable!("Unreachable code!"),
        }
    }

    /// Gives 8 bit half-index register name based on params. Passed as bool because
    /// it is easy to check bit in instruction like this: `(instruction & 0b0100) != 0`.
    pub fn reg_index_8(y_reg: bool, low: bool) -> RegName8 {
        match (y_reg, low) {
            (false, false) => RegName8::IXH,
            (false, true) => RegName8::IXL,
            (true, false) => RegName8::IYH,
            (true, true) => RegName8::IYL,
        }
    }

    /// Gives 8 bit index register name based on params.
    /// Returns IY if argument is true. Else returns IX.
    pub fn reg_index_16(y_reg: bool) -> RegName16 {
        match y_reg {
            true => RegName16::IY,
            false => RegName16::IX,
        }
    }

    fn reg_mixed_16_third_as_param(code: u8, third: RegName16) -> RegName16 {
        assert!(code <= 0b11,
                " Invalid (16 bit, mixed with HL) register index: {:#b}");
        match code {
            0b00 => RegName16::BC,
            0b01 => RegName16::DE,
            0b10 => third,
            0b11 => RegName16::SP,
            _ => unreachable!("Unreachable code!"),
        }
    }

    /// Gives 16 bit register name:
    /// - 00b => BC
    /// - 01b => DE
    /// - 10b => HL
    /// - 11b => SP
    /// Code must be lower than `0b100`
    pub fn reg_mixed_16_third_hl(code: u8) -> RegName16 {
        Self::reg_mixed_16_third_as_param(code, RegName16::HL)
    }

    /// Gives 16 bit register name:
    /// - 00b => BC
    /// - 01b => DE
    /// - 10b => IX
    /// - 11b => SP
    /// Code must be lower than `0b100`
    pub fn reg_mixed_16_third_ix(code: u8) -> RegName16 {
        Self::reg_mixed_16_third_as_param(code, RegName16::IX)
    }

    /// Gives 16 bit register name:
    /// - 00b => BC
    /// - 01b => DE
    /// - 10b => IY
    /// - 11b => SP
    /// Code must be lower than `0b100`
    pub fn reg_mixed_16_third_iy(code: u8) -> RegName16 {
        Self::reg_mixed_16_third_as_param(code, RegName16::IY)
    }

    /// Return condition type based on code. Code must be lower than `0b1000`.
    pub fn condition(code: u8) -> Condition {
        assert!(code <= 0b111, " Invalid condition index: {:#b}");
        match code {
            0b000 => Condition::NonZero,
            0b001 => Condition::Zero,
            0b010 => Condition::NonCary,
            0b011 => Condition::Cary,
            0b100 => Condition::ParityOdd,
            0b110 => Condition::ParityEven,
            0b101 => Condition::SignPositive,
            0b111 => Condition::SignNegative,
            _ => unreachable!("Unreachable code!"),
        }
    }
}

fn make16bit(hi: u8, lo: u8) -> u16 {
    ((hi as u16) << 8) | (lo as u16)
}

fn split16bit(value: u16) -> (u8, u8) {
    ((value >> 8) as u8, value as u8)
}

pub struct Regs {
    /// program counter
    pc: u16,
    /// stack pointer
    sp: u16,
    /// index register X [Ho - Lo]
    ixh: u8,
    ixl: u8,
    /// index register Y [Ho - Lo]
    iyh: u8,
    iyl: u8,
    /// Memory refresh register
    r: u8,
    /// Interrupt Page Adress register
    i: u8,
    /// general purpose regs: [A, F, B, C, D, E, H, L]
    gp: [u8; 8],
    /// Alternative general purpose regs
    gp_alt: [u8; 8],
}

/// Z80 Registers
impl Regs {
    /// Make new Regs struct
    pub fn new() -> Regs {
        Regs {
            pc: 0,
            sp: 0,
            ixh: 0,
            ixl: 0,
            iyh: 0,
            iyl: 0,
            r: 0,
            i: 0,
            gp: [0_u8; 8],
            gp_alt: [0_u8; 8],
        }
    }
}
