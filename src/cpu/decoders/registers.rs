use cpu::registers::{RegName8, RegName16};
/// Decoding cpu registers\conditions names.
pub struct RegNameDecoder;
impl RegNameDecoder {
    /// Gives 8 bit general purpose register name from code. Code must be lower than `0b1000`.
    /// # panics
    /// when code equals `0b110` (Indirect)
    pub fn reg_8(code: u8) -> RegName8 {
        assert!(code <= 0b111, " Invalid (8 bit, gp) register index: {:#b}");
        match code {
            0b111 => RegName8::A,
            0b000 => RegName8::B,
            0b001 => RegName8::C,
            0b010 => RegName8::D,
            0b011 => RegName8::E,
            0b100 => RegName8::H,
            0b101 => RegName8::L,
            0b110 => panic!("Can't decode 0b110 reg8, it is indirection"),
            _ => unreachable!(),
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
            _ => unreachable!(),
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
            _ => unreachable!(),
        }
    }
}
