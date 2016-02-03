use cpu::registers::{RegName8, RegName16};
/// Decoding cpu registers\conditions names.
pub struct RegNameDecoder;
impl RegNameDecoder {
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
    pub fn reg_16_with_af(code: u8) -> RegName16 {
        assert!(code <= 0b11, " Invalid (16 bit, gp) register index: {:#b}");
        match code {
            0b11 => RegName16::AF,
            0b00 => RegName16::BC,
            0b01 => RegName16::DE,
            0b10 => RegName16::HL,
            _ => unreachable!("Unreachable code!"),
        }
    }

    /// Gives 16 bit general purpose register name from code. Code must be lower than `0b100`
    pub fn reg_16_with_sp(code: u8) -> RegName16 {
        assert!(code <= 0b11, " Invalid (16 bit, gp) register index: {:#b}");
        match code {
            0b11 => RegName16::SP,
            0b00 => RegName16::BC,
            0b01 => RegName16::DE,
            0b10 => RegName16::HL,
            _ => unreachable!("Unreachable code!"),
        }
    }
}
